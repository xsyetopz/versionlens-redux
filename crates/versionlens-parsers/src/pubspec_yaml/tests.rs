use crate::document::test_support::extract_range;
use crate::{DocumentInput, Ecosystem, parse_document, parse_document_with_dependency_paths};

#[test]
fn parses_pubspec_yaml_dependencies() {
    let text = "\
version: 1.2.3
dependencies:
  http: ^1.2.0
  any_dep: any
  local:
    path: ./local
  repo:
    git:
      url: git@example.test/repo.git
  hosted_dep:
    version: 1.0.0
    hosted:
      name: hosted_alias
      url: https://pub.example.test
dev_dependencies:
  test: '2.0.0'
dependency_overrides:
  override_dep:
    version: 3.0.0
";
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/pubspec.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 8);
    assert_eq!(dependencies[0].ecosystem, Ecosystem::Pub);
    assert_eq!(dependencies[0].group, "version");
    assert_eq!(dependencies[0].name, "version");
    assert_eq!(dependencies[0].requirement, "1.2.3");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "1.2.3"
    );
    assert_eq!(dependencies[1].group, "dependencies");
    assert_eq!(dependencies[1].name, "http");
    assert_eq!(dependencies[1].requirement, "^1.2.0");
    assert_eq!(dependencies[2].requirement, "*");
    assert_eq!(dependencies[3].name, "local");
    assert_eq!(dependencies[3].requirement, "./local");
    assert_eq!(dependencies[4].name, "repo");
    assert_eq!(dependencies[4].requirement, "git@example.test/repo.git");
    assert_eq!(dependencies[5].name, "hosted_dep");
    assert_eq!(dependencies[5].requirement, "1.0.0");
    assert_eq!(
        dependencies[5].hosted_url.as_deref(),
        Some("https://pub.example.test")
    );
    assert_eq!(dependencies[5].hosted_name.as_deref(), Some("hosted_alias"));
    assert_eq!(dependencies[6].group, "dev_dependencies");
    assert_eq!(dependencies[6].requirement, "2.0.0");
    assert_eq!(
        extract_range(text, dependencies[6].requirement_range),
        "2.0.0"
    );
    assert_eq!(dependencies[7].group, "dependency_overrides");
    assert_eq!(dependencies[7].name, "override_dep");
}

#[test]
fn parses_pubspec_yaml_blank_versions() {
    let text = "\
dependencies:
  http: # blank with comment
  equatable:
";
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/pubspec.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].name, "http");
    assert_eq!(dependencies[0].requirement, "*");
    assert_eq!(dependencies[0].requirement_suffix, " ");
    assert_eq!(dependencies[1].name, "equatable");
    assert_eq!(dependencies[1].requirement, "");
    assert_eq!(dependencies[1].requirement_prefix, " ");
}

#[test]
fn parses_configured_pubspec_member_dependency_paths() {
    let text = "\
dependencies:
  http: ^1.2.0
  dio: ^5.0.0
dev_dependencies:
  test: ^1.25.0
";
    let dependencies = parse_document_with_dependency_paths(
        &DocumentInput {
            uri: "file:///work/pubspec.yaml".to_owned(),
            language_id: "yaml".to_owned(),
            text: text.to_owned(),
            workspace_root: None,
        },
        &[
            "dependencies.http".to_owned(),
            "dev_dependencies.*".to_owned(),
        ],
    );

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].group, "dependencies");
    assert_eq!(dependencies[0].name, "http");
    assert_eq!(dependencies[1].group, "dev_dependencies");
    assert_eq!(dependencies[1].name, "test");
}

#[test]
fn ignores_configured_pubspec_array_dependency_paths() {
    let text = "\
fonts:
  - family: SST Arabic
    fonts:
      - asset: assets/fonts/SST-Arabic-Medium.ttf
";
    let dependencies = parse_document_with_dependency_paths(
        &DocumentInput {
            uri: "file:///work/pubspec.yaml".to_owned(),
            language_id: "yaml".to_owned(),
            text: text.to_owned(),
            workspace_root: None,
        },
        &["fonts".to_owned()],
    );

    assert!(dependencies.is_empty());
}

#[test]
fn parses_smoke_pubspec_smoke_shapes() {
    let text = "\
name: testApp
description: test smoke config
version: 1.4.0
environment:
  sdk: \">=2.0.0-dev.9.4.flutter-f9ebf21297 <3.0.0\"
dependencies:
  flutter:
    sdk: flutter
  # The following adds the Cupertino Icons font to your application.
  # Use with the CupertinoIcons class for iOS style icons.
  firebase_app_check: 0.4.5
  cupertino_icons: 1.0.9
  flutter_bloc: 9.1.1
  equatable: ^2.0.8
  sqflite:
    git:
      url: https://github.com/tekartik/sqflite
      path: sqflite
  cached_network_image: 3.4.1
  http: 1.6.0 # blank entry with comment
  glob:
    version: \"2.1.3\"
  dio:
    version: 1.* # test comment
    hosted: https://pub.dev/
  http_parser:
    path: ../../

dev_dependencies:
  flutter_test:
    sdk: flutter
  build_test: 3.5.15
  test: \">=1.31.1\"
  collection: \"^1.19.1\"

dependency_overrides:
  injectable_generator: ^3.1.0
  intl_utils: ^2.8.16
  json_serializable: ^6.14.0
  mobx_codegen: any

flutter:
  uses-material-design: true
";
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/pubspec.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 18);
    assert_eq!(dependencies[0].name, "version");
    assert_eq!(dependencies[1].name, "firebase_app_check");
    assert_eq!(dependencies[5].name, "sqflite");
    assert_eq!(
        dependencies[5].requirement,
        "https://github.com/tekartik/sqflite"
    );
    assert_eq!(dependencies[8].name, "glob");
    assert_eq!(dependencies[8].requirement, "2.1.3");
    assert_eq!(dependencies[9].name, "dio");
    assert_eq!(dependencies[9].requirement, "1.*");
    assert_eq!(
        dependencies[9].hosted_url.as_deref(),
        Some("https://pub.dev/")
    );
    assert_eq!(dependencies[10].name, "http_parser");
    assert_eq!(dependencies[10].requirement, "../../");
    assert_eq!(dependencies[11].group, "dev_dependencies");
    assert_eq!(dependencies[11].name, "build_test");
    assert_eq!(dependencies[14].group, "dependency_overrides");
    assert_eq!(dependencies[17].name, "mobx_codegen");
    assert_eq!(dependencies[17].requirement, "*");
}

#[test]
fn ignores_hosted_pub_dependency_without_version_for_upstream_parity() {
    let text = "dependencies:\n  hosted_dep:\n    hosted:\n      name: hosted_alias\n      url: https://pub.example.test\n";
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/pubspec.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert!(dependencies.is_empty());
}
