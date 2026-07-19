use super::{
    DocumentInput, RegistryResponseInput, session_without_vulnerabilities, standard_session,
};
use std::fs::read_to_string;
use std::path::PathBuf;
use versionlens_model::Ecosystem::{Deno, Npm};

mod cache;
mod dist_tags;
mod http;
mod registry_config;

#[test]
fn resolves_update_from_registry_response_body() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture("resolves-update-from-registry-response-body.json"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Npm,
            body: r#"{"dist-tags":{"latest":"1.1.0"}}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "updateAvailable");
    assert_eq!(output.edits[0].new_text, "1.1.0");
}

#[test]
fn resolves_npm_alias_dependencies_against_target_package() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture(
                "resolves-npm-alias-dependencies-against-target-package.json",
            ),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "typescript".to_owned(),
            ecosystem: Npm,
            body: r#"{"dist-tags":{"latest":"6.0.4"}}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].dependency.name, "typescript");
    assert_eq!(output.suggestions[0].status, "updateAvailable");
    assert_eq!(output.edits[0].new_text, "npm:typescript@6.0.4");
}

#[test]
fn resolves_ranged_npm_alias_dependencies_preserving_range_prefix() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture(
                "resolves-ranged-npm-alias-dependencies-preserving-range-prefix.json",
            ),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "@types/react".to_owned(),
            ecosystem: Npm,
            body: r#"{"dist-tags":{"latest":"20.0.0"}}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].dependency.name, "@types/react");
    assert_eq!(output.suggestions[0].status, "updateAvailable");
    assert_eq!(output.edits[0].new_text, "npm:@types/react@^20.0.0");
}

#[test]
fn resolves_unversioned_npm_alias_dependencies_against_target_package() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture(
                "resolves-unversioned-npm-alias-dependencies-against-target-package.json",
            ),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "types-react".to_owned(),
            ecosystem: Npm,
            body: r#"{"dist-tags":{"latest":"19.2.7"}}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].dependency.name, "types-react");
    assert_eq!(output.suggestions[0].status, "updateAvailable");
    assert_eq!(output.edits[0].new_text, "npm:types-react@19.2.7");
}

#[test]
fn resolves_package_yaml_npm_alias_dependencies_preserving_alias() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///package.yaml".to_owned(),
            language_id: "yaml".to_owned(),
            text: package_file_fixture(
                "resolves-package-yaml-npm-alias-dependencies-preserving-alias.yaml",
            ),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "typescript".to_owned(),
            ecosystem: Npm,
            body: r#"{"dist-tags":{"latest":"7.0.0"}}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].dependency.name, "typescript");
    assert_eq!(output.suggestions[0].status, "updateAvailable");
    assert_eq!(output.edits[0].new_text, "npm:typescript@^7.0.0");
}

#[test]
fn resolves_deno_jsr_imports_preserving_specifier_scheme() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///deno.json".to_owned(),
            language_id: "jsonc".to_owned(),
            text: package_file_fixture(
                "resolves-deno-jsr-imports-preserving-specifier-scheme.json",
            ),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "@std/assert".to_owned(),
            ecosystem: Deno,
            body: r#"{"latest":"1.1.0","versions":{"1.0.0":{},"1.1.0":{}}}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].dependency.name, "@std/assert");
    assert_eq!(output.suggestions[0].status, "updateAvailable");
    assert_eq!(output.edits[0].new_text, "jsr:@std/assert@1.1.0");
}

#[test]
fn resolves_deno_npm_imports_preserving_specifier_scheme() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///deno.json".to_owned(),
            language_id: "jsonc".to_owned(),
            text: package_file_fixture(
                "resolves-deno-npm-imports-preserving-specifier-scheme.json",
            ),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "chalk".to_owned(),
            ecosystem: Npm,
            body: r#"{"dist-tags":{"latest":"5.4.0"}}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].dependency.name, "chalk");
    assert_eq!(output.suggestions[0].status, "updateAvailable");
    assert_eq!(output.edits[0].new_text, "npm:chalk@5.4.0");
}

#[test]
fn resolves_invalid_empty_ranges_as_invalid_range_with_latest_update() {
    let session = session_without_vulnerabilities();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture(
                "resolves-invalid-empty-ranges-as-invalid-range-with-latest-update.json",
            ),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Npm,
            body: r#"{"dist-tags":{"latest":"5.0.0"}}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "invalidRange");
    assert_eq!(output.edits[0].new_text, "5.0.0");
}

#[test]
fn missing_fixed_npm_registry_version_resolves_no_match_with_update_choices() {
    let session = session_without_vulnerabilities();
    let input = DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture(
            "missing-fixed-npm-registry-version-resolves-no-match-with-update-choices.json",
        ),
        workspace_root: None,
    };

    let output = session.resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Npm,
            body: r#"{
              "dist-tags": { "latest": "1.0.0" },
              "versions": {
                "0.5.1": {},
                "0.6.0": {},
                "1.0.0": {}
              }
            }"#
            .to_owned(),
        }],
    );

    let analysis = session.analyze_document(input);
    let titles = analysis
        .code_lenses
        .iter()
        .map(|lens| lens.title.as_str())
        .collect::<Vec<_>>();
    let arguments = analysis
        .code_lenses
        .iter()
        .skip(1)
        .map(|lens| {
            lens.arguments
                .iter()
                .skip(2)
                .map(|value| value.as_str())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    assert_eq!(output.suggestions[0].status, "noMatch");
    assert_eq!(output.suggestions[0].latest, None);
    assert!(output.edits.is_empty());
    assert_eq!(
        titles,
        [
            "⚪ no match",
            "↑  latest 1.0.0",
            "↑  minor 0.6.0",
            "↑  patch 0.5.1"
        ]
    );
    assert_eq!(
        arguments,
        [
            vec!["update", "1.0.0"],
            vec!["updateMinor", "0.6.0"],
            vec!["updatePatch", "0.5.1"]
        ]
    );
}

#[test]
fn fixed_npm_prerelease_resolves_fixed_with_prerelease_update_choice() {
    let session = session_without_vulnerabilities();
    let input = DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture(
            "fixed-npm-prerelease-resolves-fixed-with-prerelease-update-choice.json",
        ),
        workspace_root: None,
    };

    let output = session.resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Npm,
            body: r#"{
              "versions": {
                "1.0.0-beta.1": {},
                "1.0.0-beta.2": {},
                "1.0.0-beta.3": {}
              }
            }"#
            .to_owned(),
        }],
    );

    let analysis = session.analyze_document(input);
    let titles = analysis
        .code_lenses
        .iter()
        .map(|lens| lens.title.as_str())
        .collect::<Vec<_>>();
    let arguments = analysis
        .code_lenses
        .iter()
        .skip(1)
        .map(|lens| {
            lens.arguments
                .iter()
                .skip(2)
                .map(|value| value.as_str())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("1.0.0-beta.1")
    );
    assert!(output.edits.is_empty());
    assert_eq!(titles, ["🟡 fixed 1.0.0-beta.1", "↑  beta 1.0.0-beta.3"]);
    assert_eq!(arguments, [vec!["update", "1.0.0-beta.3"]]);
}

#[test]
fn fixed_npm_release_resolves_fixed_with_release_update_choices() {
    let session = session_without_vulnerabilities();
    let input = DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture(
            "fixed-npm-release-resolves-fixed-with-release-update-choices.json",
        ),
        workspace_root: None,
    };

    let output = session.resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Npm,
            body: r#"{
              "versions": {
                "1.1.0": {},
                "1.1.1": {},
                "1.1.2": {},
                "1.2.0": {},
                "1.2.2": {},
                "2.0.0": {},
                "2.2.2": {}
              }
            }"#
            .to_owned(),
        }],
    );

    let analysis = session.analyze_document(input);
    let titles = analysis
        .code_lenses
        .iter()
        .map(|lens| lens.title.as_str())
        .collect::<Vec<_>>();
    let arguments = analysis
        .code_lenses
        .iter()
        .skip(1)
        .map(|lens| {
            lens.arguments
                .iter()
                .skip(2)
                .map(|value| value.as_str())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(output.suggestions[0].latest.as_deref(), Some("1.1.1"));
    assert!(output.edits.is_empty());
    assert_eq!(
        titles,
        [
            "🟡 fixed 1.1.1",
            "↑  latest 2.2.2",
            "↑  minor 1.2.2",
            "↑  patch 1.1.2"
        ]
    );
    assert_eq!(
        arguments,
        [
            vec!["update", "2.2.2"],
            vec!["updateMinor", "1.2.2"],
            vec!["updatePatch", "1.1.2"]
        ]
    );
}

include!("npm/workspaces.rs");

fn package_file_fixture(name: &str) -> String {
    let path = repo_root()
        .join("tests/fixtures/session/resolution/tests/npm")
        .join(name);
    read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "failed to read session resolution fixture {}: {error}",
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
