use crate::{DocumentInput, Ecosystem, document::test_support::extract_range, parse_document};

#[test]
fn parses_catalog_and_named_catalogs() {
    let text = "\
catalog:
  react: ^16.14.0
catalogs:
  react17:
    react: '^17.0.2'
    react-dom: ^17.0.2
";
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/pnpm-workspace.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 3);
    assert_eq!(dependencies[0].ecosystem, Ecosystem::Npm);
    assert_eq!(dependencies[0].group, "catalog");
    assert_eq!(dependencies[0].name, "react");
    assert_eq!(dependencies[0].requirement, "^16.14.0");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "^16.14.0"
    );
    assert_eq!(dependencies[1].group, "catalogs.react17");
    assert_eq!(dependencies[1].name, "react");
    assert_eq!(dependencies[1].requirement, "^17.0.2");
    assert_eq!(
        extract_range(text, dependencies[1].requirement_range),
        "^17.0.2"
    );
    assert_eq!(dependencies[2].name, "react-dom");
}

#[test]
fn parses_yarnrc_catalog() {
    let text = "\
catalog:
  react: ^18.2.0
";
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/.yarnrc.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].ecosystem, Ecosystem::Npm);
    assert_eq!(dependencies[0].group, "catalog");
    assert_eq!(dependencies[0].name, "react");
    assert_eq!(dependencies[0].requirement, "^18.2.0");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "^18.2.0"
    );
}

#[test]
fn parses_smoke_yarnrc_smoke_shapes() {
    let text = "\
catalog:
  lodash: ^4.18.1

catalogs:
  react18:
    react: ^19.2.7
    react-dom: ^19.2.7
  react17:
    react: ^19.2.7
    react-dom: ^19.2.7
";
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/.yarnrc.yml".to_owned(),
        language_id: "yaml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 5);
    assert_eq!(dependencies[0].ecosystem, Ecosystem::Npm);
    assert_eq!(dependencies[0].group, "catalog");
    assert_eq!(dependencies[0].name, "lodash");
    assert_eq!(dependencies[0].requirement, "^4.18.1");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "^4.18.1"
    );
    assert_eq!(dependencies[1].group, "catalogs.react18");
    assert_eq!(dependencies[1].name, "react");
    assert_eq!(dependencies[2].name, "react-dom");
    assert_eq!(dependencies[3].group, "catalogs.react17");
    assert_eq!(dependencies[3].name, "react");
    assert_eq!(dependencies[4].name, "react-dom");
}

#[test]
fn parses_root_overrides() {
    let text = "\
overrides:
  vite: ^5.4.0
";
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/pnpm-workspace.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].ecosystem, Ecosystem::Npm);
    assert_eq!(dependencies[0].group, "overrides");
    assert_eq!(dependencies[0].name, "vite");
    assert_eq!(dependencies[0].requirement, "^5.4.0");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "^5.4.0"
    );
}

#[test]
fn parses_package_extensions() {
    let text = "\
packageExtensions:
  react@18:
    dependencies:
      scheduler: ^0.23.0
    peerDependencies:
      '@types/react': ^18.0.0
  vite@5:
    optionalDependencies:
      fsevents: ^2.3.3
";
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/pnpm-workspace.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 3);
    assert_eq!(dependencies[0].ecosystem, Ecosystem::Npm);
    assert_eq!(
        dependencies[0].group,
        "packageExtensions.react@18.dependencies"
    );
    assert_eq!(dependencies[0].name, "scheduler");
    assert_eq!(dependencies[0].requirement, "^0.23.0");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "^0.23.0"
    );
    assert_eq!(
        dependencies[1].group,
        "packageExtensions.react@18.peerDependencies"
    );
    assert_eq!(dependencies[1].name, "@types/react");
    assert_eq!(dependencies[1].requirement, "^18.0.0");
    assert_eq!(
        dependencies[2].group,
        "packageExtensions.vite@5.optionalDependencies"
    );
    assert_eq!(dependencies[2].name, "fsevents");
}

#[test]
fn parses_smoke_pnpm_workspace_smoke_shapes() {
    let text = "\
catalog:
  react: ^19.2.7
  react-dom: ^19.2.7
catalogs:
  react18:
    react: ^18.3.1
    react-dom: ^19.2.7
overrides:
  typescript: ^6.0.3
packageExtensions:
  react-redux:
    peerDependencies:
      react-dom: \"*\"
";
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/pnpm-workspace.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 6);
    assert_eq!(dependencies[0].ecosystem, Ecosystem::Npm);
    assert_eq!(dependencies[0].group, "catalog");
    assert_eq!(dependencies[0].name, "react");
    assert_eq!(dependencies[0].requirement, "^19.2.7");
    assert_eq!(dependencies[1].name, "react-dom");
    assert_eq!(dependencies[1].requirement, "^19.2.7");
    assert_eq!(dependencies[2].group, "overrides");
    assert_eq!(dependencies[2].name, "typescript");
    assert_eq!(dependencies[2].requirement, "^6.0.3");
    assert_eq!(dependencies[3].group, "catalogs.react18");
    assert_eq!(dependencies[3].name, "react");
    assert_eq!(dependencies[3].requirement, "^18.3.1");
    assert_eq!(dependencies[4].name, "react-dom");
    assert_eq!(
        dependencies[5].group,
        "packageExtensions.react-redux.peerDependencies"
    );
    assert_eq!(dependencies[5].name, "react-dom");
    assert_eq!(dependencies[5].requirement, "*");
    assert_eq!(extract_range(text, dependencies[5].requirement_range), "*");
}
