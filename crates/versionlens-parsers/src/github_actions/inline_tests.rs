use super::parse_github_actions;

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
