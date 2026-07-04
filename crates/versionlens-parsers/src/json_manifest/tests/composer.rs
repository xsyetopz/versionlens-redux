use super::{DocumentInput, Ecosystem, parse_document};

#[test]
fn parses_composer_json_dependency_groups() {
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/composer.json".to_owned(),
        language_id: "json".to_owned(),
        text: r#"{
  "version": "1.0.0",
  "require": {
    "php": "^8.3",
    "phpunit/phpunit": "^10.0",
    "local": { "path": "../local" },
    "remote": { "repository": "git@example.com:org/repo.git" }
  },
  "require-dev": {
    "friendsofphp/php-cs-fixer": "^3.0"
  }
}"#
        .to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 6);
    assert_eq!(dependencies[0].ecosystem, Ecosystem::Composer);
    assert_eq!(dependencies[0].group, "version");
    assert_eq!(dependencies[1].group, "require");
    assert_eq!(dependencies[1].name, "php");
    assert_eq!(dependencies[2].name, "phpunit/phpunit");
    assert_eq!(dependencies[3].requirement, "../local");
    assert_eq!(dependencies[4].requirement, "git@example.com:org/repo.git");
    assert_eq!(dependencies[5].group, "require-dev");
}

#[test]
fn parses_smoke_composer_smoke_shapes() {
    let text = r#"{
  "name": "cerzat43/smoke-test",
  "description": "smoke tests for vscode-versionlens extension",
  "version": "1.0.0",
  "require": {
    "php": "^7.1.3",
    "allocine/twigcs": "^3.1.3",
    "phpunit/phpunit": "13.2.1",
    "symfony/console": "8.1.*"
  },
  "require-dev": {
    "symfony/dotenv": "8.1.*",
    "squizlabs/php_codesniffer": "^4.0.1"
  }
}"#;
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/composer.json".to_owned(),
        language_id: "json".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 7);
    assert_eq!(dependencies[0].ecosystem, Ecosystem::Composer);
    assert_eq!(dependencies[0].group, "version");
    assert_eq!(dependencies[0].name, "1.0.0");
    assert_eq!(dependencies[1].group, "require");
    assert_eq!(dependencies[1].name, "php");
    assert_eq!(dependencies[1].requirement, "^7.1.3");
    assert_eq!(dependencies[3].name, "phpunit/phpunit");
    assert_eq!(dependencies[3].requirement, "13.2.1");
    assert_eq!(dependencies[5].group, "require-dev");
    assert_eq!(dependencies[6].name, "squizlabs/php_codesniffer");
    assert_eq!(dependencies[6].requirement, "^4.0.1");
}
