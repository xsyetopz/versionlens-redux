mod docker;
mod runtime;

use crate::positions::line_range;
use crate::{support, yaml};
use versionlens_model::Ecosystem::GitHub;
use versionlens_model::{CanonicalReference, Dependency, GithubRepository, version_tag_parts};

pub(crate) fn parse_github_actions(text: &str) -> Vec<Dependency> {
    support::with_yaml_mapping(text, |root| {
        let mut dependencies = vec![];
        let workflow_env = root.get_mapping("env");
        let outer = Scope {
            matrix: None,
            job_env: None,
            workflow_env,
        };
        if let Some(runs) = root.get_mapping("runs") {
            collect_steps(text, runs, outer, &mut dependencies);
        }
        collect_steps(text, root, outer, &mut dependencies);
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
                    let scope = Scope {
                        matrix,
                        job_env: job.get_mapping("env"),
                        workflow_env,
                    };
                    collect_steps(text, job, scope, &mut dependencies);
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

/// The expression sources of a job or workflow: `matrix` and the `env` blocks,
/// which `${{ env.X }}` reads from in step, job, then workflow order.
#[derive(Clone, Copy)]
struct Scope<'a> {
    matrix: Option<&'a marked_yaml::types::MarkedMappingNode>,
    job_env: Option<&'a marked_yaml::types::MarkedMappingNode>,
    workflow_env: Option<&'a marked_yaml::types::MarkedMappingNode>,
}

fn collect_steps(
    text: &str,
    mapping: &marked_yaml::types::MarkedMappingNode,
    scope: Scope<'_>,
    dependencies: &mut Vec<Dependency>,
) {
    if let Some(steps) = mapping.get_sequence("steps") {
        for step in steps.iter() {
            if let Some(step) = step.as_mapping() {
                runtime::collect(text, step, scope, dependencies);
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
mod inline_tests {
use super::parse_github_actions;
use versionlens_model::{CanonicalReference, Dependency};

fn runtime_dependencies(dependencies: &[Dependency]) -> Vec<&Dependency> {
    dependencies
        .iter()
        .filter(|dependency| dependency.is_runtime_version())
        .collect()
}

#[test]
fn parses_versioned_actions() {
    let dependencies =
        parse_github_actions("jobs:\n  build:\n    steps:\n      - uses: actions/checkout@v4\n");

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].name, "actions/checkout");
    assert_eq!(dependencies[0].requirement, "4");
    assert_eq!(dependencies[0].requirement_range.start.line, 3);
    assert_eq!(dependencies[0].hosted_url, None);
    assert_eq!(
        dependencies[0].hosted_name.as_deref(),
        Some("actions/checkout")
    );
}

#[test]
fn parses_reusable_workflow_references_using_the_repository_tags() {
    let dependencies = parse_github_actions(
        "jobs:\n  release:\n    uses: acme/automation/.github/workflows/release.yml@v1.2.0\n",
    );

    assert_eq!(dependencies.len(), 1);
    assert_eq!(
        dependencies[0].name,
        "acme/automation/.github/workflows/release.yml"
    );
    assert_eq!(dependencies[0].requirement, "1.2.0");
    assert_eq!(dependencies[0].hosted_url, None);
    assert_eq!(
        dependencies[0].hosted_name.as_deref(),
        Some("acme/automation")
    );
}

#[test]
fn parses_sha_pins_with_version_annotations_as_one_canonical_reference() {
    let sha = "3d3c42e5aac5ba805825da76410c181273ba90b1";
    let source = format!("steps:\n  - uses: actions/checkout@{sha} # v7.0.1\n");
    let dependencies = parse_github_actions(&source);

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].requirement, "7.0.1");
    assert_eq!(dependencies[0].requirement_prefix, "v");
    assert_eq!(
        dependencies[0].canonical_reference,
        Some(CanonicalReference::GitHubActionSha {
            commit: sha.to_owned(),
            tag: "v7.0.1".to_owned(),
            separator: " # ".to_owned(),
        })
    );
    assert_eq!(
        &source[dependency_byte_range(&source, &dependencies[0])],
        format!("{sha} # v7.0.1")
    );
}

#[test]
fn parses_quoted_abbreviated_sha_pins_and_path_qualified_tags() {
    let source =
        "steps:\n  - uses: 'acme/automation/action@3d3c42e'  # release/action-v2.4.0 kept\n";
    let dependencies = parse_github_actions(source);

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].requirement, "2.4.0");
    assert_eq!(dependencies[0].requirement_prefix, "release/action-v");
    assert_eq!(
        dependencies[0].canonical_reference,
        Some(CanonicalReference::GitHubActionSha {
            commit: "3d3c42e".to_owned(),
            tag: "release/action-v2.4.0".to_owned(),
            separator: "'  # ".to_owned(),
        })
    );
}

fn dependency_byte_range(
    source: &str,
    dependency: &versionlens_model::Dependency,
) -> std::ops::Range<usize> {
    let line_start = source
        .lines()
        .take(dependency.requirement_range.start.line as usize)
        .map(|line| line.len() + 1)
        .sum::<usize>();
    line_start + dependency.requirement_range.start.character as usize
        ..line_start + dependency.requirement_range.end.character as usize
}

#[test]
fn parses_quoted_commented_and_malformed_uses_forms_safely() {
    let dependencies = parse_github_actions(
        "jobs:\n  build:\n    steps:\n      - uses: \"actions/checkout@v4\" # checkout release\n      - uses: actions/setup-node@V4.1.0 # setup release\n  release:\n    uses: 'acme/automation/.github/workflows/release.yml@1.2.0'\n  malformed:\n    uses: actions/cache\n  empty:\n    uses: actions/cache@\n",
    );

    assert_eq!(dependencies.len(), 5);
    assert_eq!(dependencies[0].name, "actions/checkout");
    assert_eq!(dependencies[0].requirement, "4");
    assert_eq!(dependencies[1].name, "actions/setup-node");
    assert_eq!(dependencies[1].requirement, "4.1.0");
    assert_eq!(
        dependencies[2].name,
        "acme/automation/.github/workflows/release.yml"
    );
    assert_eq!(dependencies[2].requirement, "1.2.0");
    assert!(matches!(
        dependencies[3].canonical_reference,
        Some(CanonicalReference::GitHubActionInvalid { .. })
    ));
    assert!(matches!(
        dependencies[4].canonical_reference,
        Some(CanonicalReference::GitHubActionInvalid { .. })
    ));
}

#[test]
fn preserves_floating_local_and_expression_references_with_exact_ranges() {
    let source = "jobs:\n  call:\n    uses: ./.github/workflows/release.yml\n  build:\n    steps:\n      - uses: acme/action@stable\n      - uses: ./actions/build\n      - uses: ${{ matrix.action }}\n      - uses: acme/action@${{ matrix.ref }}\n";
    let dependencies = parse_github_actions(source);

    assert_eq!(dependencies.len(), 5);
    assert_eq!(
        dependencies[0].requirement,
        "./.github/workflows/release.yml"
    );
    assert_eq!(dependencies[1].requirement, "stable");
    assert_eq!(dependencies[2].requirement, "./actions/build");
    assert_eq!(dependencies[3].requirement, "${{ matrix.action }}");
    assert_eq!(dependencies[4].requirement, "${{ matrix.ref }}");
    assert_eq!(
        dependencies[0].canonical_reference,
        Some(CanonicalReference::GitHubActionLocal {
            path: "./.github/workflows/release.yml".to_owned(),
            reusable_workflow: true,
        })
    );
    assert_eq!(
        dependencies[1].canonical_reference,
        Some(CanonicalReference::GitHubActionRef {
            reference: "stable".to_owned(),
        })
    );
    assert_eq!(
        dependencies[2].canonical_reference,
        Some(CanonicalReference::GitHubActionLocal {
            path: "./actions/build".to_owned(),
            reusable_workflow: false,
        })
    );
    assert!(matches!(
        dependencies[3].canonical_reference,
        Some(CanonicalReference::GitHubActionExpression { .. })
    ));
    assert!(matches!(
        dependencies[4].canonical_reference,
        Some(CanonicalReference::GitHubActionExpression { .. })
    ));
    for dependency in &dependencies {
        assert_eq!(
            &source[dependency_byte_range(source, dependency)],
            dependency.requirement
        );
    }
}

#[test]
fn structural_actions_include_flow_mappings_and_exclude_script_and_environment_values() {
    let source = "jobs:\n  build:\n    env: {uses: acme/fake@v1}\n    steps:\n      - {uses: 'acme/action@v1.2.0', name: '🦀'}\n      - run: |\n          uses: acme/script@v9\n";
    let dependencies = parse_github_actions(source);
    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].name, "acme/action");
    assert_eq!(dependencies[0].requirement, "1.2.0");
    assert_eq!(
        &source[dependency_byte_range(source, &dependencies[0])],
        "v1.2.0"
    );
}

#[test]
fn parses_docker_action_without_changing_quotes_or_registry() {
    let source = "jobs: {build: {steps: [{uses: 'docker://ghcr.io/acme/tool:1.2.0'}]}}\n";
    let dependencies = parse_github_actions(source);
    assert_eq!(dependencies.len(), 1);
    assert_eq!(
        dependencies[0].ecosystem,
        versionlens_model::Ecosystem::Docker
    );
    assert_eq!(dependencies[0].name, "acme/tool");
    assert_eq!(dependencies[0].hosted_url.as_deref(), Some("ghcr.io"));
    assert_eq!(
        &source[dependency_byte_range(source, &dependencies[0])],
        "1.2.0"
    );
}

#[test]
fn unicode_before_a_flow_reference_uses_utf16_columns() {
    let source = "steps: [{name: '🦀', uses: 'acme/action@v1.2.0'}]";
    let dependency = parse_github_actions(source).remove(0);
    let start = source.find("v1.2.0").unwrap();
    assert_eq!(
        dependency.requirement_range.start.character as usize,
        source[..start].encode_utf16().count()
    );
    assert_eq!(
        dependency.requirement_range.end.character as usize,
        source[..start + "v1.2.0".len()].encode_utf16().count()
    );
}

#[test]
fn sha_annotation_never_includes_neighboring_flow_fields_in_an_edit() {
    let dependencies =
        parse_github_actions("steps: [{uses: acme/action@3d3c42e, name: keep}] # v1.0.0\n");
    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].requirement, "3d3c42e");
    assert_eq!(
        dependencies[0].requirement_range.end.character
            - dependencies[0].requirement_range.start.character,
        7
    );
}

#[test]
fn runtime_inputs_follow_their_action_schema_and_matrix_source_ranges() {
    let source = "jobs:\n  build:\n    strategy:\n      matrix:\n        node: ['20.0.0', '22.0.0']\n        include: [{node: '24.0.0'}]\n    steps:\n      - uses: actions/setup-node@v4\n        with: {node-version: '${{ matrix.node }}'}\n      - uses: acme/other@v1\n        with: {node-version: '99.0.0'}\n      - run: |\n          node-version: 88.0.0\n";
    let dependencies = parse_github_actions(source);
    let runtimes = runtime_dependencies(&dependencies);
    assert_eq!(
        runtimes
            .iter()
            .map(|dependency| dependency.requirement.as_str())
            .collect::<Vec<_>>(),
        ["20.0.0", "22.0.0", "24.0.0"]
    );
    for dependency in runtimes {
        assert_eq!(
            &source[dependency_byte_range(source, dependency)],
            dependency.requirement
        );
    }
}

#[test]
fn ci_runtime_schemas_preserve_context_and_multiline_version_ranges() {
    let source = "steps:\n  - uses: actions/setup-go@v6\n    with: {go-version: '1.24'}\n  - uses: denoland/setup-deno@v2\n    with: {deno-version: lts}\n  - uses: actions/setup-python@v6\n    with: {python-version: '3.13'}\n  - uses: actions/setup-java@v5\n    with:\n      distribution: temurin\n      java-version: |\n        17\n        21.0.1\n  - uses: actions/setup-dotnet@v5\n    with:\n      dotnet-version: |\n        8.0.x\n        9.0.100\n  - uses: ruby/setup-ruby@v1\n    with: {ruby-version: '3.4'}\n";
    let dependencies = parse_github_actions(source);
    let runtimes = runtime_dependencies(&dependencies);
    assert_eq!(
        runtimes
            .iter()
            .map(|dependency| (dependency.name.as_str(), dependency.requirement.as_str()))
            .collect::<Vec<_>>(),
        [
            ("go", "1.24"),
            ("deno", "lts"),
            ("python", "3.13"),
            ("java", "17"),
            ("java", "21.0.1"),
            ("dotnet", "8.0.x"),
            ("dotnet", "9.0.100"),
            ("ruby", "3.4"),
        ]
    );
    assert_eq!(runtimes[3].hosted_name.as_deref(), Some("temurin"));
    assert_eq!(runtimes[4].hosted_name.as_deref(), Some("temurin"));
    for dependency in runtimes {
        assert_eq!(
            &source[dependency_byte_range(source, dependency)],
            dependency.requirement
        );
    }
}

#[test]
fn composite_actions_include_runtime_inputs_and_rust_revision_toolchains() {
    let source = "runs:\n  using: composite\n  steps:\n    - uses: oven-sh/setup-bun@v2\n      with: {bun-version: '1.0.0'}\n    - uses: dtolnay/rust-toolchain@stable\n";
    let dependencies = parse_github_actions(source);
    let runtimes = dependencies
        .iter()
        .filter(|dependency| dependency.is_runtime_version())
        .collect::<Vec<_>>();
    assert_eq!(runtimes.len(), 2);
    assert_eq!(runtimes[0].name, "bun");
    assert_eq!(runtimes[1].name, "rust");
    assert_eq!(
        &source[dependency_byte_range(source, runtimes[1])],
        "stable"
    );
}

#[test]
fn commit_pin_ranges_preserve_quotes_and_prose_comments() {
    for comment in ["", " # pinned", " # main", " # v"] {
        let source = format!("steps:\n  - uses: 'actions/checkout@3d3c42e'{comment}\n");
        let dependencies = parse_github_actions(&source);
        assert_eq!(dependencies.len(), 1);
        assert_eq!(
            dependencies[0].canonical_reference,
            Some(CanonicalReference::GitHubActionCommit {
                commit: "3d3c42e".to_owned()
            })
        );
        assert_eq!(
            &source[dependency_byte_range(&source, &dependencies[0])],
            "3d3c42e"
        );
    }
}

#[test]
fn escaped_action_scalars_use_encoded_requirement_ranges() {
    for (reference, name, requirement, encoded) in [
        (
            r#"acme\u002faction\u0040v\u0031.2.0"#,
            "acme/action",
            "1.2.0",
            r#"v\u0031.2.0"#,
        ),
        (
            r#"docker:\/\/ghcr.io/acme/tool:1.\u0032.0"#,
            "acme/tool",
            "1.2.0",
            r#"1.\u0032.0"#,
        ),
    ] {
        let source = format!("steps:\n  - uses: \"{reference}\" # retained\n");
        let dependencies = parse_github_actions(&source);
        assert_eq!(dependencies.len(), 1, "{source}");
        assert_eq!(dependencies[0].name, name);
        assert_eq!(dependencies[0].requirement, requirement);
        assert_eq!(
            &source[dependency_byte_range(&source, &dependencies[0])],
            encoded
        );
    }
}

#[test]
fn env_references_resolve_from_step_then_job_then_workflow_env() {
    let source = "env:\n  NODE_VERSION: \"18\"\njobs:\n  build:\n    env:\n      NODE_VERSION: \"20\"\n    steps:\n      - uses: actions/setup-node@v4\n        with:\n          node-version: ${{ env.NODE_VERSION }}\n      - uses: actions/setup-node@v4\n        env:\n          NODE_VERSION: \"22\"\n        with:\n          node-version: ${{ env.NODE_VERSION }}\n  other:\n    steps:\n      - uses: actions/setup-node@v4\n        with:\n          node-version: ${{ env.NODE_VERSION }}\n";
    let dependencies = parse_github_actions(source);
    let nodes = dependencies
        .iter()
        .filter(|dependency| dependency.name == "node")
        .map(|dependency| dependency.requirement.as_str())
        .collect::<Vec<_>>();
    assert_eq!(nodes, ["18", "20", "22"]);
}

#[test]
fn unresolvable_expressions_produce_no_runtime_dependency() {
    let source = "steps:\n  - uses: actions/setup-dotnet@v4\n    with:\n      dotnet-version: ${{ steps.read.outputs.version }}\n  - uses: actions/setup-node@v4\n    with:\n      node-version: ${{ inputs.node }}\n";
    let dependencies = parse_github_actions(source);
    assert_eq!(
        runtime_dependencies(&dependencies).len(),
        0,
        "{dependencies:?}"
    );
}
}
