use serde_json::to_value;
use std::fs::read_to_string;
use std::path::PathBuf;

use versionlens_model::{DocumentInput, Ecosystem, ManifestKind};

use crate::{DependencyPropertyConfig, RegistryResponseInput, SessionConfig, VersionLensSession};
use versionlens_model::Ecosystem::{Deno, Go, Npm};
use versionlens_model::ManifestKind::NpmPackageJson;

#[test]
fn reports_sort_capability_only_for_supported_documents() {
    let session = standard_session(false);
    let package_json_catalog_session = session_with_dependency_properties(
        false,
        Npm,
        Some(NpmPackageJson),
        &["workspaces.catalogs.*"],
    );
    let input = |uri: &str, language_id: &str, text: &str| DocumentInput {
        uri: uri.to_owned(),
        language_id: language_id.to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    };

    let requirements = session.analyze_document(input(
        "file:///requirements.txt",
        "pip-requirements",
        package_file_fixture("requirements-unsorted.txt").as_str(),
    ));
    let package_json = session.analyze_document(input(
        "file:///package.json",
        "json",
        package_file_fixture("package-single-line.json").as_str(),
    ));
    let multiline_package_json = session.analyze_document(input(
        "file:///package.json",
        "json",
        package_file_fixture("package-dependencies-unsorted.json").as_str(),
    ));
    let package_json_with_metadata = session.analyze_document(input(
        "file:///package.json",
        "json",
        package_file_fixture("package-with-package-manager.json").as_str(),
    ));
    let pubspec = session.analyze_document(input(
        "file:///pubspec.yaml",
        "yaml",
        package_file_fixture("pubspec-unsorted.yaml").as_str(),
    ));
    let composer = session.analyze_document(input(
        "file:///composer.json",
        "json",
        package_file_fixture("composer-unsorted.json").as_str(),
    ));
    let pnpm_workspace = session.analyze_document(input(
        "file:///pnpm-workspace.yaml",
        "yaml",
        package_file_fixture("pnpm-workspace-catalogs-unsorted.yaml").as_str(),
    ));
    let package_json_workspace_catalogs = package_json_catalog_session.analyze_document(input(
        "file:///package.json",
        "json",
        package_file_fixture("package-workspace-catalogs-unsorted.json").as_str(),
    ));
    let maven = session.analyze_document(input(
        "file:///pom.xml",
        "xml",
        package_file_fixture("pom-unsorted.xml").as_str(),
    ));
    let dotnet = session.analyze_document(input(
        "file:///app.csproj",
        "xml",
        package_file_fixture("app-unsorted.csproj").as_str(),
    ));
    let go_mod = session.analyze_document(input(
        "file:///go.mod",
        "go.mod",
        package_file_fixture("go-unsorted.mod").as_str(),
    ));
    let empty_package_json = session.analyze_document(input(
        "file:///package.json",
        "json",
        package_file_fixture("empty-package.json").as_str(),
    ));
    let unsupported = session.analyze_document(input("file:///notes.txt", "plaintext", "hello"));

    assert!(requirements.can_sort_dependencies);
    assert!(pubspec.can_sort_dependencies);
    assert!(multiline_package_json.can_sort_dependencies);
    assert!(package_json_with_metadata.can_sort_dependencies);
    assert!(composer.can_sort_dependencies);
    assert!(pnpm_workspace.can_sort_dependencies);
    assert!(package_json_workspace_catalogs.can_sort_dependencies);
    assert!(maven.can_sort_dependencies);
    assert!(dotnet.can_sort_dependencies);
    assert!(go_mod.can_sort_dependencies);
    assert!(!package_json.can_sort_dependencies);
    assert!(empty_package_json.is_supported_manifest);
    assert!(!unsupported.is_supported_manifest);
}

#[test]
fn reports_sort_capability_for_deno_scoped_imports() {
    let session = session_with_dependency_properties(false, Deno, None, &["scopes"]);

    let output = session.analyze_document(DocumentInput {
        uri: "file:///deno.json".to_owned(),
        language_id: "jsonc".to_owned(),
        text: package_file_fixture("deno-scopes-unsorted.json"),
        workspace_root: None,
    });

    assert!(output.can_sort_dependencies);
}

#[test]
fn reports_sort_capability_for_gemfile_dependencies() {
    let session = standard_session(false);

    let output = session.analyze_document(DocumentInput {
        uri: "file:///Gemfile".to_owned(),
        language_id: "ruby".to_owned(),
        text: package_file_fixture("Gemfile-unsorted"),
        workspace_root: None,
    });

    assert!(output.can_sort_dependencies);
}

#[test]
fn analyze_document_reports_active_provider_name_for_supported_manifests() {
    let session = standard_session(false);

    let npm = session.analyze_document(package_json_input(
        package_file_fixture("empty-package.json").as_str(),
    ));
    let package_json5 = session.analyze_document(DocumentInput {
        uri: "file:///package.json5".to_owned(),
        language_id: "json5".to_owned(),
        text: package_file_fixture("package-provider.json5"),
        workspace_root: None,
    });
    let package_yaml = session.analyze_document(DocumentInput {
        uri: "file:///package.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: package_file_fixture("package-provider.yaml"),
        workspace_root: None,
    });
    let deno_import_map = session.analyze_document(DocumentInput {
        uri: "file:///import_map.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture("import-map-provider.json"),
        workspace_root: None,
    });
    let jsr = session.analyze_document(DocumentInput {
        uri: "file:///jsr.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture("jsr-provider.json"),
        workspace_root: None,
    });
    let pnpm = session.analyze_document(DocumentInput {
        uri: "file:///pnpm-workspace.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: package_file_fixture("pnpm-workspace-provider.yaml"),
        workspace_root: None,
    });
    let golang = session.analyze_document(DocumentInput {
        uri: "file:///go.mod".to_owned(),
        language_id: "go.mod".to_owned(),
        text: package_file_fixture("go-provider.mod"),
        workspace_root: None,
    });
    let pypi = session.analyze_document(DocumentInput {
        uri: "file:///requirements.txt".to_owned(),
        language_id: "pip-requirements".to_owned(),
        text: package_file_fixture("requirements-provider.txt"),
        workspace_root: None,
    });
    let ruby_gemspec = session.analyze_document(DocumentInput {
        uri: "file:///example.gemspec".to_owned(),
        language_id: "ruby".to_owned(),
        text: package_file_fixture("example-provider.gemspec"),
        workspace_root: None,
    });
    let unsupported = session.analyze_document(DocumentInput {
        uri: "file:///notes.txt".to_owned(),
        language_id: "plaintext".to_owned(),
        text: "hello".to_owned(),
        workspace_root: None,
    });

    assert_eq!(npm.active_provider_name, Some("npm".to_owned()));
    assert_eq!(package_json5.active_provider_name, Some("npm".to_owned()));
    assert_eq!(package_yaml.active_provider_name, Some("npm".to_owned()));
    assert_eq!(
        deno_import_map.active_provider_name,
        Some("deno".to_owned())
    );
    assert_eq!(jsr.active_provider_name, Some("deno".to_owned()));
    assert_eq!(pnpm.active_provider_name, Some("pnpm".to_owned()));
    assert_eq!(golang.active_provider_name, Some("golang".to_owned()));
    assert_eq!(pypi.active_provider_name, Some("pypi".to_owned()));
    assert_eq!(ruby_gemspec.active_provider_name, Some("ruby".to_owned()));
    let terraform = session.analyze_document(DocumentInput {
        uri: "file:///main.tf".to_owned(),
        language_id: "terraform".to_owned(),
        text: package_file_fixture("terraform-provider.tf"),
        workspace_root: None,
    });
    let helm = session.analyze_document(DocumentInput {
        uri: "file:///Chart.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: package_file_fixture("Chart-provider.yaml"),
        workspace_root: None,
    });
    assert_eq!(terraform.active_provider_name, Some("terraform".to_owned()));
    assert_eq!(helm.active_provider_name, Some("helm".to_owned()));
    assert_eq!(unsupported.active_provider_name, None);
}

#[test]
fn analyze_document_serializes_dependencies_as_vscode_payloads() {
    let session = standard_session(false);

    let output = session.analyze_document(DocumentInput {
        uri: "file:///go.mod".to_owned(),
        language_id: "go.mod".to_owned(),
        text: package_file_fixture("go-single-require.mod"),
        workspace_root: None,
    });
    let value = to_value(output).unwrap();

    assert_eq!(value["dependencies"][0]["name"], "example.test/pkg");
    assert_eq!(value["dependencies"][0]["ecosystem"], "golang");
}

#[test]
fn resolve_document_serializes_suggestions_as_vscode_payloads() {
    let session = standard_session(false);

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///go.mod".to_owned(),
            language_id: "go.mod".to_owned(),
            text: package_file_fixture("go-single-require.mod"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "example.test/pkg".to_owned(),
            ecosystem: Go,
            body: "v1.1.0\n".to_owned(),
        }],
    );
    let value = to_value(output).unwrap();

    assert_eq!(
        value["suggestions"][0]["dependency"]["name"],
        "example.test/pkg"
    );
    assert_eq!(value["suggestions"][0]["dependency"]["ecosystem"], "golang");
    assert_eq!(value["suggestions"][0]["status"], "updateAvailable");
}

#[test]
fn deno_non_jsr_npm_imports_produce_no_suggestions_like_upstream() {
    let session = standard_session(false);
    let input = DocumentInput {
        uri: "file:///deno.json".to_owned(),
        language_id: "jsonc".to_owned(),
        text: package_file_fixture("deno-remote-import.json"),
        workspace_root: None,
    };

    let output = session.resolve_document_with_responses(input.clone(), &[]);
    let analysis = session.analyze_document(input);

    assert_eq!(analysis.dependencies.len(), 1);
    assert!(output.suggestions.is_empty());
    assert!(analysis.code_lenses.is_empty());
    assert!(output.edits.is_empty());
}

#[test]
fn analyze_document_uses_cached_latest_for_diagnostics() {
    let mut config = standard_config(false);
    let session = crate::version_lens_session(config.clone());
    let input = package_json_input(package_file_fixture("package-single-line.json").as_str());

    session.resolve_document_with_responses(input.clone(), &[registry_response()]);

    let output = session.analyze_document(input.clone());

    assert!(output.diagnostics.is_empty());
    assert_eq!(output.status.dependency_count, 1);
    assert_eq!(output.status.update_count, 1);
    assert_eq!(output.status.vulnerability_count, 0);
    assert!(output.status.visible);
    assert_eq!(output.status.text, "$(versions) 1/1");
    assert!(!output.can_sort_dependencies);
    assert_eq!(
        output.install_task_config_key,
        Some("npm.onSaveChanges".to_owned())
    );
    assert_eq!(
        output.dependency_signature,
        concat!("npm\0left-pad\0dependencies\0", "1.0.0")
    );

    config.show_suggestion_stats = true;
    let session = crate::version_lens_session(config);
    session.resolve_document_with_responses(input.clone(), &[registry_response()]);

    assert_eq!(
        session.analyze_document(input).status.text,
        "$(versions) 1/1 updates, 0 vulnerabilities, 0 errors, 0 no matches"
    );
}

#[test]
fn analyze_document_reports_cached_errors_and_no_matches_in_status() {
    let session = standard_session(true);
    let input =
        package_json_input(package_file_fixture("package-missing-and-errored.json").as_str());

    session.resolve_document_with_responses(
        input.clone(),
        &[
            RegistryResponseInput {
                package: "missing-package".to_owned(),
                ecosystem: Npm,
                body: r#"{"versions":{}}"#.to_owned(),
            },
            RegistryResponseInput {
                package: "errored-package".to_owned(),
                ecosystem: Npm,
                body: r#"{"status":"E404"}"#.to_owned(),
            },
        ],
    );

    let output = session.analyze_document(input);

    assert_eq!(output.status.update_count, 0);
    assert_eq!(output.status.error_count, 1);
    assert_eq!(output.status.no_match_count, 1);
    assert_eq!(
        output.status.text,
        "$(versions) 0/2 updates, 0 vulnerabilities, 1 errors, 1 no matches"
    );
}

#[test]
fn analyze_document_reports_when_sort_is_unavailable() {
    let session = standard_session(false);

    let output = session.analyze_document(package_json_input(
        package_file_fixture("empty-package.json").as_str(),
    ));

    assert!(!output.can_sort_dependencies);
    assert_eq!(output.install_task_config_key, None);
    assert_eq!(output.dependency_signature, "");
}

#[test]
fn analyze_document_uses_manifest_for_install_task_key() {
    let session = standard_session(false);

    let output = session.analyze_document(DocumentInput {
        uri: "file:///deno.json".to_owned(),
        language_id: "jsonc".to_owned(),
        text: package_file_fixture("deno-npm-import.json"),
        workspace_root: None,
    });

    assert_eq!(
        output.install_task_config_key,
        Some("deno.onSaveChanges".to_owned())
    );
}

#[test]
fn analyze_document_keeps_package_json_install_task_on_npm_provider() {
    let session = standard_session(false);

    let output = session.analyze_document(DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture("package-pnpm-manager.json"),
        workspace_root: None,
    });

    assert_eq!(
        output.install_task_config_key,
        Some("npm.onSaveChanges".to_owned())
    );
}

#[test]
fn analyze_document_does_not_offer_install_task_for_pnpm_yaml() {
    let session = standard_session(false);

    let output = session.analyze_document(DocumentInput {
        uri: "file:///pnpm-workspace.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: package_file_fixture("pnpm-workspace-catalog.yaml"),
        workspace_root: None,
    });

    assert_eq!(output.dependencies.len(), 1);
    assert_eq!(output.install_task_config_key, None);
}

fn package_file_fixture(name: &str) -> String {
    let path = repo_root()
        .join("tests/fixtures/session/documents")
        .join(name);
    read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "failed to read session documents fixture {}: {error}",
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

fn standard_session(show_suggestion_stats: bool) -> VersionLensSession {
    crate::version_lens_session(standard_config(show_suggestion_stats))
}

fn standard_config(show_suggestion_stats: bool) -> SessionConfig {
    SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: crate::default(),
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities: true,
        show_suggestion_stats,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    }
}

fn session_with_dependency_properties(
    show_suggestion_stats: bool,
    ecosystem: Ecosystem,
    manifest_kind: Option<ManifestKind>,
    properties: &[&str],
) -> VersionLensSession {
    let mut config = standard_config(show_suggestion_stats);
    config.providers.dependency_properties = vec![DependencyPropertyConfig {
        ecosystem,
        manifest_kind,
        properties: properties
            .iter()
            .map(|property| (*property).to_owned())
            .collect(),
    }];
    crate::version_lens_session(config)
}

fn package_json_input(text: &str) -> DocumentInput {
    DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    }
}

fn registry_response() -> RegistryResponseInput {
    RegistryResponseInput {
        package: "left-pad".to_owned(),
        ecosystem: Npm,
        body: r#"{"dist-tags":{"latest":"1.1.0"}}"#.to_owned(),
    }
}
