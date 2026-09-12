mod docker;
mod runtime;

use crate::positions::line_range;
use crate::{support, yaml};
use versionlens_model::Ecosystem::GitHub;
use versionlens_model::{CanonicalReference, Dependency, GithubRepository};
use versionlens_versions::version_tag_parts;

pub(crate) fn parse_github_actions(text: &str) -> Vec<Dependency> {
    support::with_yaml_mapping(text, |root| {
        let mut dependencies = vec![];
        if let Some(runs) = root.get_mapping("runs") {
            collect_steps(text, runs, None, &mut dependencies);
        }
        collect_steps(text, root, None, &mut dependencies);
        if let Some(jobs) = root.get_mapping("jobs") {
            for (_, job) in jobs.iter() {
                if let Some(job) = job.as_mapping() {
                    let matrix = job
                        .get_mapping("strategy")
                        .and_then(|strategy| strategy.get_mapping("matrix"));
                    if let Some(value) = job.get_scalar("uses")
                        && let Some(dependency) =
                            parse_uses_scalar(text, value, UsesKind::ReusableWorkflow)
                    {
                        dependencies.push(dependency);
                    }
                    collect_steps(text, job, matrix, &mut dependencies);
                }
            }
        }
        dependencies.sort_by_key(|dependency| {
            (
                dependency.requirement_range.start.line,
                dependency.requirement_range.start.character,
            )
        });
        dependencies
    })
    .unwrap_or_default()
}

#[derive(Clone, Copy)]
enum UsesKind {
    Action,
    ReusableWorkflow,
}

struct UsesSource<'text, 'value> {
    text: &'text str,
    value: &'value str,
    span: std::ops::Range<usize>,
    line_start: usize,
    line_index: usize,
    line: &'text str,
    value_start: usize,
    revision_end: usize,
    value_with_comment_start: usize,
    comment_start: Option<usize>,
}

struct ParsedRemoteReference {
    requirement: String,
    prefix: String,
    end: usize,
    canonical: CanonicalReference,
}

struct ReferenceDependencyInput<'a> {
    name: &'a str,
    requirement: &'a str,
    name_start: usize,
    requirement_start: usize,
    requirement_end: usize,
    hosted_name: Option<String>,
    canonical_reference: CanonicalReference,
}

fn collect_steps(
    text: &str,
    mapping: &marked_yaml::types::MarkedMappingNode,
    matrix: Option<&marked_yaml::types::MarkedMappingNode>,
    dependencies: &mut Vec<Dependency>,
) {
    if let Some(steps) = mapping.get_sequence("steps") {
        for step in steps.iter() {
            if let Some(step) = step.as_mapping() {
                runtime::collect(text, step, matrix, dependencies);
                if !runtime::revision_is_toolchain(step)
                    && let Some(value) = step.get_scalar("uses")
                    && let Some(dependency) = parse_uses_scalar(text, value, UsesKind::Action)
                {
                    dependencies.push(dependency);
                }
            }
        }
    }
}

fn parse_uses_scalar(
    text: &str,
    scalar: &marked_yaml::types::MarkedScalarNode,
    kind: UsesKind,
) -> Option<Dependency> {
    let source = uses_source(text, scalar)?;
    let value = source.value;
    if let Some(image) = value.strip_prefix("docker://") {
        return docker::dependency(text, image, source.span);
    }
    if value.starts_with("./") {
        return Some(reference_dependency(
            &source,
            ReferenceDependencyInput {
                name: value,
                requirement: value,
                name_start: source.value_start,
                requirement_start: source.value_start,
                requirement_end: source.revision_end,
                hosted_name: None,
                canonical_reference: CanonicalReference::GitHubActionLocal {
                    path: value.to_owned(),
                    reusable_workflow: matches!(kind, UsesKind::ReusableWorkflow),
                },
            },
        ));
    }
    if value.contains("${{") {
        return Some(expression_dependency(&source));
    }
    parse_remote_dependency(&source)
}

fn uses_source<'text, 'value>(
    text: &'text str,
    scalar: &'value marked_yaml::types::MarkedScalarNode,
) -> Option<UsesSource<'text, 'value>> {
    let span = yaml::scalar_range(text, scalar)?;
    let value = scalar.as_str();
    if text.get(span.clone())?.contains(['\n', '\r']) || value.contains(['\n', '\r']) {
        return None;
    }
    let line_start = text[..span.start]
        .rfind('\n')
        .map_or(0, |offset| offset + 1);
    let line_index = text[..line_start]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count();
    let line = text[line_start..].lines().next()?;
    let value_start = span.start - line_start;
    let quoted = matches!(
        text.as_bytes().get(span.start.wrapping_sub(1)),
        Some(b'\'' | b'"')
    );
    let revision_end = span.end - line_start;
    let value_with_comment_start = span.end - line_start + usize::from(quoted);
    let value_with_comment = &line[value_with_comment_start..];
    let comment_start = value_with_comment
        .trim_start()
        .starts_with('#')
        .then(|| value_with_comment.len() - value_with_comment.trim_start().len());
    Some(UsesSource {
        text,
        value,
        span,
        line_start,
        line_index,
        line,
        value_start,
        revision_end,
        value_with_comment_start,
        comment_start,
    })
}

fn expression_dependency(source: &UsesSource<'_, '_>) -> Dependency {
    let value = source.value;
    let (name, requirement, requirement_start) =
        value
            .split_once('@')
            .map_or((value, value, source.value_start), |(name, requirement)| {
                (
                    name.trim(),
                    requirement.trim(),
                    source.value_start + name.len() + 1,
                )
            });
    reference_dependency(
        source,
        ReferenceDependencyInput {
            name,
            requirement,
            name_start: source.value_start,
            requirement_start,
            requirement_end: source.revision_end,
            hosted_name: github_repository_name(name).map(str::to_owned),
            canonical_reference: CanonicalReference::GitHubActionExpression {
                expression: value.to_owned(),
            },
        },
    )
}

fn parse_remote_dependency(source: &UsesSource<'_, '_>) -> Option<Dependency> {
    let value = source.value;
    let Some((name, raw_requirement)) = value.split_once('@') else {
        return Some(reference_dependency(
            source,
            invalid_reference_input(source),
        ));
    };
    let name = name.trim();
    let raw_requirement = raw_requirement.trim();
    let Some(repository) = github_repository_name(name) else {
        return Some(reference_dependency(
            source,
            invalid_reference_input(source),
        ));
    };
    let name_end = yaml::scalar_content_offset(source.text, source.span.clone(), name.len())?
        - source.line_start;
    let requirement_start =
        yaml::scalar_content_offset(source.text, source.span.clone(), name.len() + 1)?
            - source.line_start;
    let parsed = parse_remote_reference(source, raw_requirement)?;
    Some(Dependency {
        name: name.to_owned(),
        requirement: parsed.requirement,
        ecosystem: GitHub,
        group: "uses".to_owned(),
        hosted_url: None,
        hosted_name: Some(repository.to_owned()),
        range: line_range(source.line_index, source.line, source.value_start, name_end),
        requirement_range: line_range(
            source.line_index,
            source.line,
            requirement_start,
            parsed.end,
        ),
        requirement_prefix: parsed.prefix,
        requirement_suffix: "".to_owned(),
        canonical_reference: Some(parsed.canonical),
    })
}

fn invalid_reference_input<'a>(source: &'a UsesSource<'_, '_>) -> ReferenceDependencyInput<'a> {
    ReferenceDependencyInput {
        name: source.value,
        requirement: source.value,
        name_start: source.value_start,
        requirement_start: source.value_start,
        requirement_end: source.revision_end,
        hosted_name: None,
        canonical_reference: CanonicalReference::GitHubActionInvalid {
            reference: source.value.to_owned(),
        },
    }
}

fn parse_remote_reference(
    source: &UsesSource<'_, '_>,
    raw_requirement: &str,
) -> Option<ParsedRemoteReference> {
    if is_commit_sha(raw_requirement) {
        let annotation = source.comment_start.and_then(|start| {
            let comment = &source.line[source.value_with_comment_start + start + 1..];
            let trimmed = comment.trim_start();
            let annotation = trimmed.split_whitespace().next()?;
            let annotation_start =
                source.value_with_comment_start + start + 1 + comment.len() - trimmed.len();
            version_tag_parts(annotation).map(|parts| (annotation, annotation_start, parts))
        });
        if let Some((annotation, annotation_start, (prefix, requirement))) = annotation {
            return Some(ParsedRemoteReference {
                requirement: requirement.to_owned(),
                prefix: prefix.to_owned(),
                end: annotation_start + annotation.len(),
                canonical: CanonicalReference::GitHubActionSha {
                    commit: raw_requirement.to_owned(),
                    tag: annotation.to_owned(),
                    separator: source
                        .line
                        .get(source.revision_end..annotation_start)?
                        .to_owned(),
                },
            });
        }
        return Some(ParsedRemoteReference {
            requirement: raw_requirement.to_owned(),
            prefix: String::new(),
            end: source.revision_end,
            canonical: CanonicalReference::GitHubActionCommit {
                commit: raw_requirement.to_owned(),
            },
        });
    }
    let (requirement, prefix, canonical) = version_tag_parts(raw_requirement).map_or_else(
        || {
            let canonical = if raw_requirement.is_empty() {
                CanonicalReference::GitHubActionInvalid {
                    reference: source.value.to_owned(),
                }
            } else {
                CanonicalReference::GitHubActionRef {
                    reference: raw_requirement.to_owned(),
                }
            };
            (raw_requirement, "", canonical)
        },
        |(prefix, requirement)| {
            (
                requirement,
                prefix,
                CanonicalReference::GitHubActionTag {
                    tag: raw_requirement.to_owned(),
                },
            )
        },
    );
    Some(ParsedRemoteReference {
        requirement: requirement.to_owned(),
        prefix: prefix.to_owned(),
        end: source.revision_end,
        canonical,
    })
}

fn reference_dependency(
    source: &UsesSource<'_, '_>,
    input: ReferenceDependencyInput<'_>,
) -> Dependency {
    Dependency {
        name: input.name.to_owned(),
        requirement: input.requirement.to_owned(),
        ecosystem: GitHub,
        group: "uses".to_owned(),
        hosted_url: None,
        hosted_name: input.hosted_name,
        range: line_range(
            source.line_index,
            source.line,
            input.name_start,
            input.requirement_end,
        ),
        requirement_range: line_range(
            source.line_index,
            source.line,
            input.requirement_start,
            input.requirement_end,
        ),
        requirement_prefix: String::new(),
        requirement_suffix: String::new(),
        canonical_reference: Some(input.canonical_reference),
    }
}

fn github_repository_name(value: &str) -> Option<&str> {
    let mut parts = value.split('/');
    let Some(owner) = parts.next() else {
        return None;
    };
    let Some(repository) = parts.next() else {
        return None;
    };
    let identity = format!("{owner}/{repository}");
    if parts.any(|part| part.is_empty() || part == "." || part == "..") {
        return None;
    }
    GithubRepository::parse(&identity)
        .is_some()
        .then_some(&value[..owner.len() + 1 + repository.len()])
}

fn is_commit_sha(value: &str) -> bool {
    (7..=40).contains(&value.len()) && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}
#[cfg(test)]
mod inline_tests;
