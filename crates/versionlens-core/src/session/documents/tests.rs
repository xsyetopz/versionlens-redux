use versionlens_http::HttpConfig;
use versionlens_parsers::{DocumentInput, Ecosystem, ManifestKind};

use crate::{
    DependencyPropertyConfig, ProviderSettings, RegistryResponseInput, SessionConfig,
    SuggestionIndicators, VersionLensSession,
};

#[test]
fn reports_sort_capability_only_for_supported_documents() {
    let session = standard_session(false);
    let package_json_catalog_session = session_with_dependency_properties(
        false,
        Ecosystem::Npm,
        Some(ManifestKind::NpmPackageJson),
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
        "zeta==1\nalpha==1\n",
    ));
    let package_json = session.analyze_document(input(
        "file:///package.json",
        "json",
        r#"{"dependencies":{"left-pad":"1.0.0"}}"#,
    ));
    let multiline_package_json = session.analyze_document(input(
        "file:///package.json",
        "json",
        "{\n  \"dependencies\": {\n    \"zeta\": \"1\",\n    \"alpha\": \"1\"\n  }\n}",
    ));
    let package_json_with_metadata = session.analyze_document(input(
        "file:///package.json",
        "json",
        "{\n  \"version\": \"1.0.0\",\n  \"packageManager\": \"pnpm@9.0.0\",\n  \"dependencies\": {\n    \"zeta\": \"1\",\n    \"alpha\": \"1\"\n  }\n}",
    ));
    let pubspec = session.analyze_document(input(
        "file:///pubspec.yaml",
        "yaml",
        "dependencies:\n  zeta: 1\n  alpha: 1\n",
    ));
    let composer = session.analyze_document(input(
        "file:///composer.json",
        "json",
        "{\n  \"require\": {\n    \"zeta/pkg\": \"1\",\n    \"alpha/pkg\": \"1\"\n  }\n}",
    ));
    let pnpm_workspace = session.analyze_document(input(
        "file:///pnpm-workspace.yaml",
        "yaml",
        "catalogs:\n  react18:\n    react-dom: ^19.2.7\n    react: ^18.3.1\n",
    ));
    let package_json_workspace_catalogs = package_json_catalog_session.analyze_document(input(
        "file:///package.json",
        "json",
        "{\n  \"workspaces\": {\n    \"catalogs\": {\n      \"react18\": {\n        \"react-dom\": \"^19.2.7\",\n        \"react\": \"^18.3.1\"\n      }\n    }\n  }\n}",
    ));
    let maven = session.analyze_document(input(
        "file:///pom.xml",
        "xml",
        "<project>\n  <dependencies>\n    <dependency>\n      <groupId>org.zeta</groupId>\n      <artifactId>zeta</artifactId>\n      <version>1</version>\n    </dependency>\n    <dependency>\n      <groupId>org.alpha</groupId>\n      <artifactId>alpha</artifactId>\n      <version>1</version>\n    </dependency>\n  </dependencies>\n</project>",
    ));
    let dotnet = session.analyze_document(input(
        "file:///app.csproj",
        "xml",
        "<Project>\n  <ItemGroup>\n    <PackageReference Include=\"Zeta.Package\" Version=\"1\" />\n    <PackageReference Include=\"Alpha.Package\" Version=\"1\" />\n  </ItemGroup>\n</Project>",
    ));
    let go_mod = session.analyze_document(input(
        "file:///go.mod",
        "go.mod",
        "module example.test/app\n\nrequire (\n\tzeta.example/pkg v1.0.0\n\talpha.example/pkg v1.0.0\n)\n",
    ));
    let empty_package_json = session.analyze_document(input("file:///package.json", "json", "{}"));
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
    let session = session_with_dependency_properties(false, Ecosystem::Deno, None, &["scopes"]);

    let output = session.analyze_document(DocumentInput {
        uri: "file:///deno.json".to_owned(),
        language_id: "jsonc".to_owned(),
        text: "{\n  \"scopes\": {\n    \"https://deno.land/x/app/\": {\n      \"zeta\": \"jsr:@scope/zeta@1.0.0\",\n      \"alpha\": \"jsr:@scope/alpha@1.0.0\"\n    }\n  }\n}"
            .to_owned(),
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
        text: "gem \"zeta\", \"1.0.0\"\ngem \"alpha\", \"1.0.0\"\n".to_owned(),
        workspace_root: None,
    });

    assert!(output.can_sort_dependencies);
}

#[test]
fn analyze_document_reports_active_provider_name_for_supported_manifests() {
    let session = standard_session(false);

    let npm = session.analyze_document(package_json_input("{}"));
    let pnpm = session.analyze_document(DocumentInput {
        uri: "file:///pnpm-workspace.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: "packages:\n  - packages/*\n".to_owned(),
        workspace_root: None,
    });
    let golang = session.analyze_document(DocumentInput {
        uri: "file:///go.mod".to_owned(),
        language_id: "go.mod".to_owned(),
        text: "module example.test/app\n".to_owned(),
        workspace_root: None,
    });
    let pypi = session.analyze_document(DocumentInput {
        uri: "file:///requirements.txt".to_owned(),
        language_id: "pip-requirements".to_owned(),
        text: "requests==2.32.0\n".to_owned(),
        workspace_root: None,
    });
    let unsupported = session.analyze_document(DocumentInput {
        uri: "file:///notes.txt".to_owned(),
        language_id: "plaintext".to_owned(),
        text: "hello".to_owned(),
        workspace_root: None,
    });

    assert_eq!(npm.active_provider_name, Some("npm".to_owned()));
    assert_eq!(pnpm.active_provider_name, Some("pnpm".to_owned()));
    assert_eq!(golang.active_provider_name, Some("golang".to_owned()));
    assert_eq!(pypi.active_provider_name, Some("pypi".to_owned()));
    assert_eq!(unsupported.active_provider_name, None);
}

#[test]
fn analyze_document_serializes_dependencies_as_vscode_payloads() {
    let session = standard_session(false);

    let output = session.analyze_document(DocumentInput {
        uri: "file:///go.mod".to_owned(),
        language_id: "go.mod".to_owned(),
        text: "module example.test/app\n\nrequire example.test/pkg v1.0.0\n".to_owned(),
        workspace_root: None,
    });
    let value = serde_json::to_value(output).unwrap();

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
            text: "module example.test/app\n\nrequire example.test/pkg v1.0.0\n".to_owned(),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "example.test/pkg".to_owned(),
            ecosystem: Ecosystem::Go,
            body: "v1.1.0\n".to_owned(),
        }],
    );
    let value = serde_json::to_value(output).unwrap();

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
        text: r#"{"imports":{"remote":"https://deno.land/std/mod.ts"}}"#.to_owned(),
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
    let session = VersionLensSession::new(config.clone());
    let input = package_json_input(r#"{"dependencies":{"left-pad":"1.0.0"}}"#);

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
    let session = VersionLensSession::new(config);
    session.resolve_document_with_responses(input.clone(), &[registry_response()]);

    assert_eq!(
        session.analyze_document(input).status.text,
        "$(versions) 1/1 updates, 0 vulnerabilities, 0 errors, 0 no matches"
    );
}

#[test]
fn analyze_document_reports_cached_errors_and_no_matches_in_status() {
    let session = standard_session(true);
    let input = package_json_input(
        r#"{"dependencies":{"missing-package":"1.0.0","errored-package":"1.0.0"}}"#,
    );

    session.resolve_document_with_responses(
        input.clone(),
        &[
            RegistryResponseInput {
                package: "missing-package".to_owned(),
                ecosystem: Ecosystem::Npm,
                body: r#"{"versions":{}}"#.to_owned(),
            },
            RegistryResponseInput {
                package: "errored-package".to_owned(),
                ecosystem: Ecosystem::Npm,
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

    let output = session.analyze_document(package_json_input("{}"));

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
        text: r#"{"imports":{"chalk":"npm:chalk@5.3.0"}}"#.to_owned(),
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
        text: "{\"packageManager\":\"pnpm@10.0.0\",\"dependencies\":{\"react\":\"18.0.0\"}}"
            .to_owned(),
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
        text: "catalog:\n  react: ^18.0.0\n".to_owned(),
        workspace_root: None,
    });

    assert_eq!(output.dependencies.len(), 1);
    assert_eq!(output.install_task_config_key, None);
}

fn standard_session(show_suggestion_stats: bool) -> VersionLensSession {
    VersionLensSession::new(standard_config(show_suggestion_stats))
}

fn standard_config(show_suggestion_stats: bool) -> SessionConfig {
    SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: Vec::new(),
        providers: ProviderSettings::default(),
        suggestion_indicators: SuggestionIndicators::standard(),
        show_vulnerabilities: true,
        show_suggestion_stats,
        show_prereleases: false,
        http: HttpConfig::standard(),
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
    VersionLensSession::new(config)
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
        ecosystem: Ecosystem::Npm,
        body: r#"{"dist-tags":{"latest":"1.1.0"}}"#.to_owned(),
    }
}
