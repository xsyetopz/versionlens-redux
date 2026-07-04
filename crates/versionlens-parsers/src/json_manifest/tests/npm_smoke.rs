use super::{DocumentInput, Ecosystem, parse_document};
use crate::document::test_support::extract_range;

#[test]
fn parses_smoke_npm_range_smoke_shapes() {
    let text = r#"{
  "dependencies": {
    "@faker-js/faker": "> 10.0.0 < 10.5.0",
    "typescript": "> 6.0.0 < 6.0.3"
  }
}"#;
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/package.json".to_owned(),
        language_id: "json".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].ecosystem, Ecosystem::Npm);
    assert_eq!(dependencies[0].group, "dependencies");
    assert_eq!(dependencies[0].name, "@faker-js/faker");
    assert_eq!(dependencies[0].requirement, "> 10.0.0 < 10.5.0");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "> 10.0.0 < 10.5.0"
    );
    assert_eq!(dependencies[1].name, "typescript");
    assert_eq!(dependencies[1].requirement, "> 6.0.0 < 6.0.3");
}

#[test]
fn parses_smoke_jspm_package_json_smoke_shapes() {
    let text = r#"{
  "name": "smoke-test",
  "jspm": {
    "dependencies": {
      "bluebird": "npm:bluebird@^3.7.2",
      "webpack": "npm:webpack@*",
      "bootstrap": "github:twbs/bootstrap#v5.3.8",
      "css": "npm:systemjs-plugin-css@^0.1.37",
      "es6-shim": "github:es-shims/es6-shim#0.35.8"
    },
    "devDependencies": {
      "core-js": "npm:core-js@^3.49.0"
    }
  }
}"#;
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/package.json".to_owned(),
        language_id: "json".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 6);
    assert_eq!(dependencies[0].group, "jspm.dependencies");
    assert_eq!(dependencies[0].name, "bluebird");
    assert_eq!(dependencies[0].requirement, "^3.7.2");
    assert_eq!(dependencies[1].requirement, "*");
    assert_eq!(dependencies[2].name, "twbs/bootstrap");
    assert_eq!(dependencies[2].requirement, "v5.3.8");
    assert_eq!(dependencies[2].requirement_prefix, "github:twbs/bootstrap#");
    assert_eq!(dependencies[5].group, "jspm.devDependencies");
    assert_eq!(dependencies[5].name, "core-js");
    assert_eq!(dependencies[5].requirement, "^3.49.0");
}

#[test]
fn parses_smoke_typical_package_json_smoke_shapes() {
    let text = r#"{
  "name": "smoke-test",
  "devDependencies": {
    "typescript": "^6.0.3",
    "aliased": "npm:typescript@6.0.3",
    "bootstrap": "twbs/bootstrap#v5.3.8",
    "express": "expressjs/express#semver:v5.2.1",
    "test": "file:../../..",
    "@types/angular": "~1.8.9",
    "@types/node": "latest",
    "@angular/core": "22.0.3",
    "webpack": "5.108.0",
    "semver": "7.8.5",
    "@openapitools/openapi-generator-cli": "2.39.0",
    "npm": ">=11.17.0",
    "invalid": ">=4.5.",
    "@could-not-be-found/a": "^1.0.0"
  },
  "customDependencies": {
    "@types/hammerjs": "2.0.33"
  }
}"#;
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/package.json".to_owned(),
        language_id: "json".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 14);
    assert_eq!(dependencies[1].name, "typescript");
    assert_eq!(dependencies[1].requirement, "6.0.3");
    assert_eq!(dependencies[2].name, "twbs/bootstrap");
    assert_eq!(dependencies[2].requirement, "v5.3.8");
    assert_eq!(dependencies[3].name, "expressjs/express");
    assert_eq!(dependencies[3].requirement, "v5.2.1");
    assert_eq!(
        dependencies[3].requirement_prefix,
        "expressjs/express#semver:"
    );
    assert_eq!(dependencies[6].name, "@types/node");
    assert_eq!(dependencies[6].requirement, "latest");
    assert_eq!(dependencies[12].name, "invalid");
    assert_eq!(dependencies[12].requirement, ">=4.5.");
    assert!(!dependencies.iter().any(|dependency| {
        dependency.group == "customDependencies" && dependency.name == "@types/hammerjs"
    }));
}

#[test]
fn parses_smoke_npm_workspaces_smoke_shapes() {
    let text = r#"{
  "dependencies": {
    "react": "catalog:"
  },
  "workspaces": {
    "packages": [
      "packages/*"
    ],
    "catalog": {
      "react": "^19.2.7",
      "react-dom": "^19.2.7"
    },
    "catalogs": {
      "testing": {
        "jest": "30.0.0",
        "testing-library": "14.0.0"
      }
    }
  }
}"#;
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/package.json".to_owned(),
        language_id: "json".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert!(dependencies.is_empty());
}

#[test]
fn parses_smoke_npm_overrides_smoke_shapes() {
    let text = r#"{
  "overrides": {
    "semver": "7.8.5",
    "somepackage": {
      "typescript": "6.0.3",
      "semver": "7.8.5"
    }
  }
}"#;
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/package.json".to_owned(),
        language_id: "json".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 3);
    assert_eq!(dependencies[0].ecosystem, Ecosystem::Npm);
    assert_eq!(dependencies[0].group, "overrides");
    assert_eq!(dependencies[0].name, "semver");
    assert_eq!(dependencies[0].requirement, "7.8.5");
    assert_eq!(dependencies[1].group, "overrides");
    assert_eq!(dependencies[1].name, "typescript");
    assert_eq!(dependencies[2].name, "semver");
}
