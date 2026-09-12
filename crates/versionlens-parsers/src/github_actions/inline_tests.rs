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
