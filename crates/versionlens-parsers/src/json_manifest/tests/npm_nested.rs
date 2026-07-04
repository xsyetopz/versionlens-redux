use super::{DocumentInput, Ecosystem, parse_document, parse_document_with_dependency_paths};
use crate::document::test_support::extract_range;

#[test]
fn parses_package_json_nested_wildcard_and_scalars() {
    let text = r#"{
  "version": "1.2.3",
  "packageManager": "pnpm@9.1.2",
  "overrides": {
    "react@18.0.0": "18.2.0",
    "@scope/pkg@1.2.3": "1.2.4",
    "parent": {
      "child": "2.0.0"
    }
  },
  "jspm": {
    "dependencies": {
      "systemjs": "6.0.0"
    }
  },
  "pnpm": {
    "overrides": {
      "nested": {
        "leaf": "1.0.0"
      }
    }
  },
  "workspaces": {
    "catalog": {
      "react-dom": "^19.2.7"
    },
    "catalogs": {
      "testing": {
        "jest": "30.0.0"
      }
    }
  }
}"#;
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/package.json".to_owned(),
        language_id: "jsonc".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 7);
    assert_eq!(dependencies[0].group, "version");
    assert_eq!(dependencies[0].name, "1.2.3");
    assert_eq!(dependencies[1].group, "packageManager");
    assert_eq!(dependencies[1].name, "pnpm");
    assert_eq!(dependencies[1].requirement, "9.1.2");
    assert_eq!(
        extract_range(text, dependencies[1].requirement_range),
        "9.1.2"
    );
    assert_eq!(dependencies[2].name, "react");
    assert_eq!(dependencies[3].name, "@scope/pkg@1.2.3");
    assert_eq!(dependencies[4].group, "overrides");
    assert_eq!(dependencies[4].name, "child");
    assert_eq!(dependencies[5].group, "jspm.dependencies");
    assert_eq!(dependencies[6].group, "pnpm.overrides");
    assert_eq!(dependencies[6].name, "leaf");
}

#[test]
fn parses_configured_package_json_pnpm_package_extensions() {
    let text = r#"{
  "pnpm": {
    "packageExtensions": {
      "react-redux": {
        "peerDependencies": {
          "react-dom": "*"
        }
      },
      "vite@5": {
        "optionalDependencies": {
          "fsevents": "^2.3.3"
        }
      }
    }
  }
}"#;
    let dependencies = parse_document_with_dependency_paths(
        &DocumentInput {
            uri: "file:///work/package.json".to_owned(),
            language_id: "json".to_owned(),
            text: text.to_owned(),
            workspace_root: None,
        },
        &[
            "pnpm.packageExtensions.*.dependencies",
            "pnpm.packageExtensions.*.devDependencies",
            "pnpm.packageExtensions.*.peerDependencies",
            "pnpm.packageExtensions.*.optionalDependencies",
        ],
    );

    assert_eq!(dependencies.len(), 2);
    assert_eq!(
        dependencies[0].group,
        "pnpm.packageExtensions.react-redux.peerDependencies"
    );
    assert_eq!(dependencies[0].name, "react-dom");
    assert_eq!(dependencies[0].requirement, "*");
    assert_eq!(extract_range(text, dependencies[0].requirement_range), "*");
    assert_eq!(
        dependencies[1].group,
        "pnpm.packageExtensions.vite@5.optionalDependencies"
    );
    assert_eq!(dependencies[1].name, "fsevents");
    assert_eq!(dependencies[1].requirement, "^2.3.3");
}

#[test]
fn parses_configured_package_json_workspace_catalogs() {
    let text = r#"{
  "dependencies": {
    "react": "catalog:"
  },
  "workspaces": {
    "catalog": {
      "react": "^19.2.7",
      "react-dom": "^19.2.7"
    },
    "catalogs": {
      "testing": {
        "jest": "30.0.0"
      }
    }
  }
}"#;
    let dependencies = parse_document_with_dependency_paths(
        &DocumentInput {
            uri: "file:///work/package.json".to_owned(),
            language_id: "json".to_owned(),
            text: text.to_owned(),
            workspace_root: None,
        },
        &[
            "dependencies",
            "workspaces.catalog",
            "workspaces.catalogs.*",
        ],
    );

    assert_eq!(dependencies.len(), 3);
    assert_eq!(dependencies[0].group, "workspaces.catalog");
    assert_eq!(dependencies[0].name, "react");
    assert_eq!(dependencies[1].name, "react-dom");
    assert_eq!(dependencies[2].group, "workspaces.catalogs.testing");
    assert_eq!(dependencies[2].name, "jest");
}

#[test]
fn parses_configured_smoke_npm_custom_dependency_paths() {
    let text = r#"{
  "devDependencies": {
    "typescript": "^6.0.3"
  },
  "customDependencies": {
    "@types/hammerjs": "2.0.33"
  }
}"#;
    let dependencies = parse_document_with_dependency_paths(
        &DocumentInput {
            uri: "file:///work/package.json".to_owned(),
            language_id: "json".to_owned(),
            text: text.to_owned(),
            workspace_root: None,
        },
        &["customDependencies".to_owned()],
    );

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].ecosystem, Ecosystem::Npm);
    assert_eq!(dependencies[0].group, "customDependencies");
    assert_eq!(dependencies[0].name, "@types/hammerjs");
    assert_eq!(dependencies[0].requirement, "2.0.33");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "2.0.33"
    );
}
