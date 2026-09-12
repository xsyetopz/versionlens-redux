#[test]
fn parses_versioned_github_actions() {
    let dependencies = parse_document(&DocumentInput::new(
        "file:///work/.github/workflows/ci.yml".to_owned(),
        "yaml".to_owned(),
        package_file_fixture("parses-github-actions-version-refs.yaml").to_owned(),
        None,
    ));

    assert_eq!(dependencies.len(), 3);
    assert!(
        dependencies
            .iter()
            .all(|dependency| dependency.ecosystem == GitHub)
    );
    assert_eq!(dependencies[0].name, "actions/checkout");
    assert_eq!(dependencies[0].requirement, "4");
    assert_eq!(
        dependencies[0].hosted_name.as_deref(),
        Some("actions/checkout")
    );
    assert_eq!(dependencies[1].requirement, "4.1.0");
    assert_eq!(
        dependencies[1].hosted_name.as_deref(),
        Some("actions/setup-node")
    );
    assert_eq!(
        dependencies[2].name,
        "acme/automation/.github/workflows/release.yml"
    );
    assert_eq!(dependencies[2].requirement, "1.2.0");
    assert_eq!(
        dependencies[2].hosted_name.as_deref(),
        Some("acme/automation")
    );
}

#[test]
fn parses_each_structural_github_actions_reference_kind() {
    let input = DocumentInput::new(
        "file:///work/.github/workflows/ci.yml".to_owned(),
        "yaml".to_owned(),
        package_file_fixture("parses-github-actions-reference-kinds.yaml").to_owned(),
        Some("/work".to_owned()),
    );
    let dependencies = parse_document(&input);

    assert_eq!(dependencies.len(), 4);
    assert!(matches!(
        dependencies[0].canonical_reference,
        Some(versionlens_model::CanonicalReference::GitHubActionLocal {
            reusable_workflow: true,
            ..
        })
    ));
    assert!(matches!(
        dependencies[1].canonical_reference,
        Some(versionlens_model::CanonicalReference::GitHubActionRef { .. })
    ));
    assert!(matches!(
        dependencies[2].canonical_reference,
        Some(versionlens_model::CanonicalReference::GitHubActionLocal {
            reusable_workflow: false,
            ..
        })
    ));
    assert!(matches!(
        dependencies[3].canonical_reference,
        Some(versionlens_model::CanonicalReference::GitHubActionExpression { .. })
    ));
}
