use marked_yaml::types::{MarkedMappingNode, MarkedScalarNode};
use versionlens_model::{Dependency, Ecosystem, Range};

use super::Scope;
use crate::{positions::offset_range, yaml};

struct RuntimeInput {
    action: &'static str,
    input: &'static str,
    name: &'static str,
    ecosystem: Ecosystem,
    context: RuntimeContext,
    multiline: bool,
}

#[derive(Clone, Copy)]
enum RuntimeContext {
    None,
    JavaDistribution,
    Dotnet,
}

const INPUTS: &[RuntimeInput] = &[
    RuntimeInput {
        action: "actions/setup-node",
        input: "node-version",
        name: "node",
        ecosystem: Ecosystem::Npm,
        context: RuntimeContext::None,
        multiline: false,
    },
    RuntimeInput {
        action: "oven-sh/setup-bun",
        input: "bun-version",
        name: "bun",
        ecosystem: Ecosystem::Npm,
        context: RuntimeContext::None,
        multiline: false,
    },
    RuntimeInput {
        action: "actions/setup-go",
        input: "go-version",
        name: "go",
        ecosystem: Ecosystem::Go,
        context: RuntimeContext::None,
        multiline: false,
    },
    RuntimeInput {
        action: "denoland/setup-deno",
        input: "deno-version",
        name: "deno",
        ecosystem: Ecosystem::Deno,
        context: RuntimeContext::None,
        multiline: false,
    },
    RuntimeInput {
        action: "actions/setup-python",
        input: "python-version",
        name: "python",
        ecosystem: Ecosystem::Python,
        context: RuntimeContext::None,
        multiline: false,
    },
    RuntimeInput {
        action: "actions/setup-java",
        input: "java-version",
        name: "java",
        ecosystem: Ecosystem::Maven,
        context: RuntimeContext::JavaDistribution,
        multiline: true,
    },
    RuntimeInput {
        action: "actions/setup-dotnet",
        input: "dotnet-version",
        name: "dotnet",
        ecosystem: Ecosystem::Dotnet,
        context: RuntimeContext::Dotnet,
        multiline: true,
    },
    RuntimeInput {
        action: "ruby/setup-ruby",
        input: "ruby-version",
        name: "ruby",
        ecosystem: Ecosystem::Ruby,
        context: RuntimeContext::None,
        multiline: false,
    },
    RuntimeInput {
        action: "dtolnay/rust-toolchain",
        input: "toolchain",
        name: "rust",
        ecosystem: Ecosystem::Cargo,
        context: RuntimeContext::None,
        multiline: false,
    },
    RuntimeInput {
        action: "actions-rust-lang/setup-rust-toolchain",
        input: "toolchain",
        name: "rust",
        ecosystem: Ecosystem::Cargo,
        context: RuntimeContext::None,
        multiline: false,
    },
];

pub(super) fn collect(
    text: &str,
    action: &MarkedMappingNode,
    scope: Scope<'_>,
    dependencies: &mut Vec<Dependency>,
) {
    let Some(uses) = action.get_scalar("uses") else {
        return;
    };
    let Some((repository, revision)) = uses.as_str().split_once('@') else {
        return;
    };
    let Some(schema) = INPUTS
        .iter()
        .find(|schema| schema.action.eq_ignore_ascii_case(repository))
    else {
        return;
    };
    let inputs = action.get_mapping("with");
    if let Some(value) = inputs.and_then(|inputs| inputs.get_scalar(schema.input)) {
        let Some(value) = env_value(value, action, scope) else {
            return;
        };
        let Ok(context) = runtime_context(inputs, schema.context, action, scope) else {
            return;
        };
        for scalar in matrix_values(value, scope.matrix) {
            for dependency in scalar_dependencies(text, scalar, schema, context.as_deref()) {
                if !dependencies.contains(&dependency) {
                    dependencies.push(dependency);
                }
            }
        }
    } else if repository.eq_ignore_ascii_case("dtolnay/rust-toolchain")
        && (matches!(revision, "stable" | "beta" | "nightly")
            || versionlens_model::normalized_version(revision).is_some()
            || revision.starts_with("nightly-"))
        && let Some(mut dependency) = scalar_dependency(text, uses, schema, None)
        && let Some(span) = yaml::scalar_range(text, uses)
        && let Some(start) = yaml::scalar_content_offset(text, span.clone(), repository.len() + 1)
    {
        dependency.requirement = revision.to_owned();
        dependency.requirement_range = offset_range(text, start, span.end);
        dependencies.push(dependency);
    }
}

pub(super) fn revision_is_toolchain(action: &MarkedMappingNode) -> bool {
    let Some(uses) = action.get_scalar("uses") else {
        return false;
    };
    let Some((repository, revision)) = uses.as_str().split_once('@') else {
        return false;
    };
    repository.eq_ignore_ascii_case("dtolnay/rust-toolchain")
        && action
            .get_mapping("with")
            .and_then(|inputs| inputs.get_scalar("toolchain"))
            .is_none()
        && (matches!(revision, "stable" | "beta" | "nightly")
            || versionlens_model::normalized_version(revision).is_some()
            || revision.starts_with("nightly-"))
}

/// The text between `${{` and `}}` when the whole value is one expression.
fn expression_body(value: &str) -> Option<&str> {
    value
        .trim()
        .strip_prefix("${{")
        .and_then(|value| value.strip_suffix("}}"))
        .map(str::trim)
}

/// Resolves `${{ env.NAME }}` to the `env` literal that defines NAME, searching the
/// step, job, then workflow `env` blocks. Returns `None` when no scalar defines it.
/// Any other value is returned unchanged.
fn env_value<'a>(
    value: &'a MarkedScalarNode,
    action: &'a MarkedMappingNode,
    scope: Scope<'a>,
) -> Option<&'a MarkedScalarNode> {
    let Some(name) = expression_body(value.as_str()).and_then(|body| body.strip_prefix("env."))
    else {
        return Some(value);
    };
    [action.get_mapping("env"), scope.job_env, scope.workflow_env]
        .into_iter()
        .flatten()
        .find_map(|env| env.get_node(name))
        .and_then(|node| node.as_scalar())
}

fn matrix_values<'a>(
    value: &'a MarkedScalarNode,
    matrix: Option<&'a MarkedMappingNode>,
) -> Vec<&'a MarkedScalarNode> {
    let Some(key) = expression_body(value.as_str())
        .and_then(|value| value.strip_prefix("matrix."))
        .filter(|key| {
            !key.is_empty()
                && key.chars().all(|character| {
                    character.is_ascii_alphanumeric() || matches!(character, '_' | '-')
                })
        })
    else {
        return vec![value];
    };
    let Some(matrix) = matrix else {
        return vec![value];
    };
    let mut values = matrix
        .get_sequence(key)
        .into_iter()
        .flat_map(|values| values.iter())
        .filter_map(|value| value.as_scalar())
        .collect::<Vec<_>>();
    if let Some(include) = matrix.get_sequence("include") {
        values.extend(
            include
                .iter()
                .filter_map(|row| row.as_mapping().and_then(|row| row.get_scalar(key))),
        );
    }
    if values.is_empty() {
        vec![value]
    } else {
        values
    }
}

fn scalar_dependencies(
    text: &str,
    scalar: &MarkedScalarNode,
    schema: &RuntimeInput,
    context: Option<&str>,
) -> Vec<Dependency> {
    let value = scalar.as_str();
    if !schema.multiline || !value.contains(['\n', '\r']) {
        return scalar_dependency(text, scalar, schema, context)
            .into_iter()
            .collect();
    }
    let Some(span) = yaml::scalar_range(text, scalar) else {
        return vec![];
    };
    let Some(source) = text.get(span.start..) else {
        return vec![];
    };
    let mut offset = 0;
    let mut dependencies = Vec::new();
    for requirement in value.lines().map(str::trim).filter(|line| !line.is_empty()) {
        let Some(relative) = source.get(offset..).and_then(|tail| tail.find(requirement)) else {
            return scalar_dependency(text, scalar, schema, context)
                .into_iter()
                .collect();
        };
        let start = span.start + offset + relative;
        let end = start + requirement.len();
        offset += relative + requirement.len();
        if !requirement.contains("${{") {
            dependencies.push(runtime_dependency(
                offset_range(text, start, end),
                requirement,
                schema,
                context,
                true,
            ));
        }
    }
    dependencies
}

fn scalar_dependency(
    text: &str,
    scalar: &MarkedScalarNode,
    schema: &RuntimeInput,
    context: Option<&str>,
) -> Option<Dependency> {
    let value = scalar.as_str();
    if value.contains("${{") {
        return None;
    }
    let span = yaml::scalar_range(text, scalar)?;
    let literal = yaml::scalar_content_offset(text, span.clone(), value.len()) == Some(span.end)
        && !value.contains(['\n', '\r']);
    Some(runtime_dependency(
        offset_range(text, span.start, span.end),
        value,
        schema,
        context,
        literal,
    ))
}

fn runtime_dependency(
    range: Range,
    value: &str,
    schema: &RuntimeInput,
    context: Option<&str>,
    literal: bool,
) -> Dependency {
    Dependency {
        name: schema.name.to_owned(),
        requirement: value.to_owned(),
        ecosystem: schema.ecosystem,
        group: format!("with.{}", schema.input),
        hosted_url: Some(
            if literal {
                "toolchain"
            } else {
                "runtime-expression"
            }
            .to_owned(),
        ),
        hosted_name: context.map(str::to_owned),
        range,
        requirement_range: range,
        requirement_prefix: String::new(),
        requirement_suffix: String::new(),
        canonical_reference: None,
    }
}

/// A runtime context that is set but cannot be read statically.
struct UnresolvedContext;

fn runtime_context(
    inputs: Option<&MarkedMappingNode>,
    context: RuntimeContext,
    action: &MarkedMappingNode,
    scope: Scope<'_>,
) -> Result<Option<String>, UnresolvedContext> {
    match context {
        RuntimeContext::None => Ok(None),
        RuntimeContext::JavaDistribution => context_scalar(inputs, "distribution", action, scope),
        RuntimeContext::Dotnet => {
            let channel =
                context_scalar(inputs, "dotnet-channel", action, scope)?.unwrap_or_default();
            let quality =
                context_scalar(inputs, "dotnet-quality", action, scope)?.unwrap_or_default();
            Ok((!channel.is_empty() || !quality.is_empty())
                .then(|| format!("channel={channel};quality={quality}")))
        }
    }
}

fn context_scalar(
    inputs: Option<&MarkedMappingNode>,
    name: &str,
    action: &MarkedMappingNode,
    scope: Scope<'_>,
) -> Result<Option<String>, UnresolvedContext> {
    let Some(scalar) = inputs.and_then(|inputs| inputs.get_scalar(name)) else {
        return Ok(None);
    };
    let scalar = env_value(scalar, action, scope).ok_or(UnresolvedContext)?;
    let value = scalar.as_str();
    if value.contains("${{") || value.contains(['\n', '\r']) {
        return Err(UnresolvedContext);
    }
    Ok(Some(value.to_owned()))
}
