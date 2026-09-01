use super::parse_github_actions;
use versionlens_model::CanonicalReference;

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
fn ignores_local_actions_sha_refs_and_branch_refs() {
    let dependencies = parse_github_actions(
        "steps:\n  - uses: ./local-action\n  - uses: actions/checkout@main\n  - uses: actions/checkout@0123456789abcdef0123456789abcdef01234567\n",
    );

    assert!(dependencies.is_empty());
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

#[test]
fn ignores_sha_pins_without_a_single_version_bearing_annotation() {
    for annotation in ["", " # pinned", " # main", " # v"] {
        let source = format!("steps:\n  - uses: actions/checkout@3d3c42e{annotation}\n");
        assert!(parse_github_actions(&source).is_empty(), "{annotation}");
    }
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

    assert_eq!(dependencies.len(), 3);
    assert_eq!(dependencies[0].name, "actions/checkout");
    assert_eq!(dependencies[0].requirement, "4");
    assert_eq!(dependencies[1].name, "actions/setup-node");
    assert_eq!(dependencies[1].requirement, "4.1.0");
    assert_eq!(
        dependencies[2].name,
        "acme/automation/.github/workflows/release.yml"
    );
    assert_eq!(dependencies[2].requirement, "1.2.0");
}
