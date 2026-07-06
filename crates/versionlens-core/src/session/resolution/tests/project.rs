use std::env::temp_dir;
use std::fs::create_dir_all;
use std::fs::read_to_string;
use std::fs::remove_dir_all;
use std::fs::write;
use std::path::PathBuf;
use std::process::id;

use super::{DocumentInput, standard_session};
use versionlens_parsers::Ecosystem::Composer;

#[test]
fn resolves_project_version_without_registry_response() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///Cargo.toml".to_owned(),
            language_id: "toml".to_owned(),
            text: package_file_fixture("resolves-project-version-without-registry-response.toml"),
            workspace_root: None,
        },
        &[],
    );

    assert_eq!(output.suggestions[0].status, "updateAvailable");
    assert_eq!(output.edits[0].new_text, "1.2.4");
}

#[test]
fn resolves_jsr_project_version_without_registry_response() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///jsr.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture(
                "resolves-jsr-project-version-without-registry-response.json",
            ),
            workspace_root: None,
        },
        &[],
    );

    assert_eq!(output.suggestions[0].status, "updateAvailable");
    assert_eq!(output.suggestions[0].dependency.name, "@scope/pkg");
    assert_eq!(output.edits[0].new_text, "1.2.4");
}

#[test]
fn resolves_gleam_project_version_without_registry_response() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///gleam.toml".to_owned(),
            language_id: "toml".to_owned(),
            text: package_file_fixture(
                "resolves-gleam-project-version-without-registry-response.toml",
            ),
            workspace_root: None,
        },
        &[],
    );

    assert_eq!(output.suggestions[0].status, "updateAvailable");
    assert_eq!(output.suggestions[0].dependency.name, "my_package");
    assert_eq!(output.suggestions[0].dependency.group, "version");
    assert_eq!(output.edits[0].new_text, "1.2.4");
}

#[test]
fn analyzes_project_version_code_lens_without_registry_response() {
    let session = standard_session();

    let output = session.analyze_document(DocumentInput {
        uri: "file:///pubspec.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: package_file_fixture(
            "analyzes-project-version-code-lens-without-registry-response.yaml",
        ),
        workspace_root: None,
    });

    let titles = output
        .code_lenses
        .iter()
        .map(|lens| lens.title.as_str())
        .collect::<Vec<_>>();
    let commands = output
        .code_lenses
        .iter()
        .filter_map(|lens| lens.arguments.get(2).map(|value| value.as_str()))
        .collect::<Vec<_>>();

    assert_eq!(titles.len(), 3);
    assert!(titles.iter().any(|title| title.contains("major 2.0.0")));
    assert!(titles.iter().any(|title| title.contains("minor 1.3.0")));
    assert!(titles.iter().any(|title| title.contains("patch 1.2.4")));
    assert!(titles.iter().all(|title| !title.contains(" available")));
    assert_eq!(commands, ["updateMajor", "updateMinor", "updatePatch"]);
}

#[test]
fn composer_repositories_override_registry_urls() {
    let input = DocumentInput {
        uri: "file:///repo/composer.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture("composer-repositories-override-registry-urls.json"),
        workspace_root: None,
    };
    let context = crate::registry::registry_context_from_document(&input);
    let dependencies = standard_session().dependencies(&input);
    let session = standard_session();

    assert_eq!(dependencies[0].name, "phpunit/phpunit");
    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec!["https://composer.example.test/phpunit/phpunit.json"]
    );
}

#[test]
fn composer_auth_json_supplies_request_scoped_auth_headers() {
    let root = temp_dir().join(format!("versionlens-composer-auth-{}", id()));
    create_dir_all(&root).unwrap();
    write(
        root.join("auth.json"),
        r#"{
  "http-basic": {
    "composer.example.test": {"username":"user","password":"pass"},
    "composer.example.test/private": {"username":"scoped","password":"secret"}
  },
  "bearer": {
    "bearer.example.test": "token"
  }
}"#,
    )
    .unwrap();

    let input = DocumentInput {
        uri: format!("file://{}", root.join("composer.json").display()),
        language_id: "json".to_owned(),
        text: package_file_fixture("composer-auth-json-supplies-request-scoped-auth-headers.txt"),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = crate::registry::registry_context_from_document(&input);
    let default_headers = context.auth_headers_for_url(
        Composer,
        "https://composer.example.test/p2/phpunit/phpunit.json",
    );
    let scoped_headers = context.auth_headers_for_url(
        Composer,
        "https://composer.example.test/private/p2/phpunit/phpunit.json",
    );
    let bearer_headers = context.auth_headers_for_url(
        Composer,
        "https://bearer.example.test/p2/phpunit/phpunit.json",
    );
    let other_headers = context.auth_headers_for_url(
        Composer,
        "https://other.example.test/p2/phpunit/phpunit.json",
    );

    assert_eq!(default_headers.len(), 1);
    assert_eq!(default_headers[0].value, "Basic dXNlcjpwYXNz");
    assert_eq!(scoped_headers.len(), 1);
    assert_eq!(scoped_headers[0].value, "Basic c2NvcGVkOnNlY3JldA==");
    assert_eq!(bearer_headers.len(), 1);
    assert_eq!(bearer_headers[0].value, "Bearer token");
    assert!(other_headers.is_empty());

    remove_dir_all(root).unwrap();
}

fn package_file_fixture(name: &str) -> String {
    let path = repo_root()
        .join("tests/fixtures/session/resolution/project")
        .join(name);
    read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "failed to read session resolution project fixture {}: {error}",
            path.display()
        )
    })
}

fn repo_root() -> PathBuf {
    let manifest_dir: PathBuf = env!("CARGO_MANIFEST_DIR").into();
    manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .expect("core crate should be under crates/")
        .to_path_buf()
}
