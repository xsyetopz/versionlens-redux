use crate::{DependencyPropertyConfig, RegistryResponseInput, SessionConfig, VersionLensSession};
use serde_json::to_value;

use versionlens_model::{DocumentInput, Ecosystem, ManifestKind};

use versionlens_model::Ecosystem::*;
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
    let input = |uri: &str, language_id: &str, text: &str| {
        DocumentInput::new(
            uri.to_owned(),
            language_id.to_owned(),
            text.to_owned(),
            None,
        )
    };

    let requirements = session.analyze_document(input(
        "file:///requirements.txt",
        "pip-requirements",
        package_file_fixture("requirements-unsorted.txt").as_str(),
    ));
    let package_json = session.analyze_document(input(
        "file:///package.json",
        "json",
        package_file_fixture("single-line.json").as_str(),
    ));
    let multiline_package_json = session.analyze_document(input(
        "file:///package.json",
        "json",
        package_file_fixture("dependencies-unsorted.json").as_str(),
    ));
    let package_json_with_metadata = session.analyze_document(input(
        "file:///package.json",
        "json",
        package_file_fixture("with-package-tool.json").as_str(),
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
        package_file_fixture("workspace-catalogs-unsorted.yaml").as_str(),
    ));
    let package_json_workspace_catalogs = package_json_catalog_session.analyze_document(input(
        "file:///package.json",
        "json",
        package_file_fixture("workspace-catalogs-unsorted.json").as_str(),
    ));
    let maven = session.analyze_document(input(
        "file:///pom.xml",
        "xml",
        crate::support::tests::fixture(
            "tests/fixtures/session/commands/sort",
            "dependencies-unsorted.xml",
        )
        .as_str(),
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

    let output = session.analyze_document(DocumentInput::new(
        "file:///deno.json".to_owned(),
        "jsonc".to_owned(),
        package_file_fixture("scopes-unsorted.json"),
        None,
    ));

    assert!(output.can_sort_dependencies);
}

#[test]
fn reports_sort_capability_for_gemfile_dependencies() {
    let session = standard_session(false);

    let output = session.analyze_document(DocumentInput::new(
        "file:///Gemfile".to_owned(),
        "ruby".to_owned(),
        package_file_fixture("Gemfile-unsorted"),
        None,
    ));

    assert!(output.can_sort_dependencies);
}

#[test]
fn analyze_document_reports_active_provider_name_for_supported_manifests() {
    let session = standard_session(false);

    let npm = session.analyze_document(package_json_input(
        package_file_fixture("empty-package.json").as_str(),
    ));
    let package_json5 = session.analyze_document(DocumentInput::new(
        "file:///package.json5".to_owned(),
        "json5".to_owned(),
        package_file_fixture("provider.json5"),
        None,
    ));
    let package_yaml = session.analyze_document(DocumentInput::new(
        "file:///package.yaml".to_owned(),
        "yaml".to_owned(),
        package_file_fixture("provider.yaml"),
        None,
    ));
    let deno_import_map = session.analyze_document(DocumentInput::new(
        "file:///import_map.json".to_owned(),
        "json".to_owned(),
        package_file_fixture("import-map-provider.json"),
        None,
    ));
    let jsr = session.analyze_document(DocumentInput::new(
        "file:///jsr.json".to_owned(),
        "json".to_owned(),
        package_file_fixture("jsr-provider.json"),
        None,
    ));
    let pnpm = session.analyze_document(DocumentInput::new(
        "file:///pnpm-workspace.yaml".to_owned(),
        "yaml".to_owned(),
        package_file_fixture("workspace-provider.yaml"),
        None,
    ));
    let golang = session.analyze_document(DocumentInput::new(
        "file:///go.mod".to_owned(),
        "go.mod".to_owned(),
        package_file_fixture("go-provider.mod"),
        None,
    ));
    let pypi = session.analyze_document(DocumentInput::new(
        "file:///requirements.txt".to_owned(),
        "pip-requirements".to_owned(),
        package_file_fixture("requirements-provider.txt"),
        None,
    ));
    let ruby_gemspec = session.analyze_document(DocumentInput::new(
        "file:///example.gemspec".to_owned(),
        "ruby".to_owned(),
        package_file_fixture("example-provider.gemspec"),
        None,
    ));
    let unsupported = session.analyze_document(DocumentInput::new(
        "file:///notes.txt".to_owned(),
        "plaintext".to_owned(),
        "hello".to_owned(),
        None,
    ));

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
    let terraform = session.analyze_document(DocumentInput::new(
        "file:///main.tf".to_owned(),
        "terraform".to_owned(),
        package_file_fixture("terraform-provider.tf"),
        None,
    ));
    let helm = session.analyze_document(DocumentInput::new(
        "file:///Chart.yaml".to_owned(),
        "yaml".to_owned(),
        package_file_fixture("Chart-provider.yaml"),
        None,
    ));
    assert_eq!(terraform.active_provider_name, Some("terraform".to_owned()));
    assert_eq!(helm.active_provider_name, Some("helm".to_owned()));
    assert_eq!(unsupported.active_provider_name, None);
}

#[test]
fn analyze_document_serializes_dependencies_as_vscode_payloads() {
    let session = standard_session(false);

    let output = session.analyze_document(DocumentInput::new(
        "file:///go.mod".to_owned(),
        "go.mod".to_owned(),
        package_file_fixture("go-single-require.mod"),
        None,
    ));
    let value = to_value(output).unwrap();

    assert_eq!(value["dependencies"][0]["name"], "example.test/pkg");
    assert_eq!(value["dependencies"][0]["ecosystem"], "golang");
}

#[test]
fn resolve_document_serializes_suggestions_as_vscode_payloads() {
    let session = standard_session(false);

    let output = session.resolve_document_with_responses(
        DocumentInput::new(
            "file:///go.mod".to_owned(),
            "go.mod".to_owned(),
            package_file_fixture("go-single-require.mod"),
            None,
        ),
        &[RegistryResponseInput::new(
            "example.test/pkg".to_owned(),
            Go,
            "v1.1.0\n".to_owned(),
        )],
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
    let input = DocumentInput::new(
        "file:///deno.json".to_owned(),
        "jsonc".to_owned(),
        package_file_fixture("remote-import.json"),
        None,
    );

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
    let input = package_json_input(package_file_fixture("single-line.json").as_str());

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
    let input = package_json_input(package_file_fixture("missing-and-errored.json").as_str());

    let output = crate::support::tests::analyze_with_responses(
        &session,
        &input,
        &[
            RegistryResponseInput::new(
                "missing-package".to_owned(),
                Npm,
                r#"{"versions":{}}"#.to_owned(),
            ),
            RegistryResponseInput::new(
                "errored-package".to_owned(),
                Npm,
                r#"{"status":"E404"}"#.to_owned(),
            ),
        ],
    );

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

    let output = session.analyze_document(DocumentInput::new(
        "file:///deno.json".to_owned(),
        "jsonc".to_owned(),
        package_file_fixture("npm-import.json"),
        None,
    ));

    assert_eq!(
        output.install_task_config_key,
        Some("deno.onSaveChanges".to_owned())
    );
}

#[test]
fn analyze_document_keeps_package_json_install_task_on_npm_provider() {
    let session = standard_session(false);

    let output = session.analyze_document(DocumentInput::new(
        "file:///package.json".to_owned(),
        "json".to_owned(),
        package_file_fixture("pnpm-tool.json"),
        None,
    ));

    assert_eq!(
        output.install_task_config_key,
        Some("npm.onSaveChanges".to_owned())
    );
}

#[test]
fn analyze_document_does_not_offer_install_task_for_pnpm_yaml() {
    let session = standard_session(false);

    let output = session.analyze_document(DocumentInput::new(
        "file:///pnpm-workspace.yaml".to_owned(),
        "yaml".to_owned(),
        package_file_fixture("workspace-catalog.yaml"),
        None,
    ));

    assert_eq!(output.dependencies.len(), 1);
    assert_eq!(output.install_task_config_key, None);
}

fn package_file_fixture(name: &str) -> String {
    crate::support::tests::fixture("tests/fixtures/session/documents", name)
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
    DocumentInput::new(
        "file:///package.json".to_owned(),
        "json".to_owned(),
        text.to_owned(),
        None,
    )
}

fn registry_response() -> RegistryResponseInput {
    RegistryResponseInput::new(
        "left-pad".to_owned(),
        Npm,
        r#"{"dist-tags":{"latest":"1.1.0"}}"#.to_owned(),
    )
}
