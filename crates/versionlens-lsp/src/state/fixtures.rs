//! The group 2.2 regression fixtures, checked through the server. They share
//! manifests and registry answers with the engine fixtures
//! (`versionlens_test_support::regression`) and expect the same lenses and
//! diagnostics, so every client of the server (`JetBrains`, Zed, Neovim) gets
//! the engine's behavior.

use lsp_types::{CodeLens, Diagnostic, NumberOrString};
use versionlens_engine::{RegistryResponseInput, Settings, UPDATE_AVAILABLE};
use versionlens_test_support::regression::{self, Case};

use super::{VersionLensLspState, VersionLensTextDocument};

struct Served {
    lenses: Vec<CodeLens>,
    diagnostics: Vec<Diagnostic>,
}

impl Served {
    fn titles(&self) -> Vec<&str> {
        self.lenses
            .iter()
            .filter_map(|lens| lens.command.as_ref())
            .map(|command| command.title.as_str())
            .collect()
    }

    fn bumps(&self) -> Vec<&str> {
        self.titles()
            .into_iter()
            .filter(|title| title.starts_with('↑'))
            .collect()
    }

    /// The versions the bump lenses offer, from their command arguments. The
    /// server keeps the range operator when it applies one.
    fn bump_versions(&self) -> Vec<String> {
        let mut versions = self
            .lenses
            .iter()
            .filter_map(|lens| lens.command.as_ref())
            .filter(|command| command.title.starts_with('↑'))
            .filter_map(|command| command.arguments.as_ref()?.first()?.get("version"))
            .filter_map(|version| version.as_str().map(str::to_owned))
            .collect::<Vec<_>>();
        versions.sort();
        versions.dedup();
        versions
    }

    /// A bump lens sits on the given line, where the edit lands.
    fn assert_bump_on_line(&self, line: u32) {
        assert!(
            self.lenses.iter().any(|lens| {
                lens.range.start.line == line
                    && lens
                        .command
                        .as_ref()
                        .is_some_and(|command| command.title.starts_with('↑'))
            }),
            "no bump on line {line}: {:?}",
            self.lenses
        );
    }

    fn assert_status(&self, status: &str) {
        assert!(
            self.titles()
                .iter()
                .any(|title| title.ends_with(status) && !title.starts_with('↑')),
            "expected status `{status}`, lenses: {:?}",
            self.titles()
        );
    }

    fn assert_no_bump(&self) {
        assert_eq!(self.bumps(), Vec::<&str>::new());
    }

    /// No diagnostic other than an available update.
    fn assert_silent(&self) {
        let noise = self
            .diagnostics
            .iter()
            .filter(|diagnostic| {
                diagnostic.code != Some(NumberOrString::String(UPDATE_AVAILABLE.to_owned()))
            })
            .collect::<Vec<_>>();
        assert_eq!(noise, Vec::<&Diagnostic>::new());
    }
}

/// Registries that refuse connections, so any lookup surfaces as an error
/// instead of reaching the network.
fn offline() -> serde_json::Value {
    let unreachable = serde_json::json!({"apiUrl": "http://127.0.0.1:9/"});
    serde_json::json!({
        "github": unreachable,
        "npm": unreachable,
        "dotnet": {"nuget": {"sources": ["http://127.0.0.1:9/"]}},
    })
}

/// Opens the fixture in the server and resolves it with its registry answers.
async fn serve(case: &Case, offline_registries: bool) -> Served {
    let mut settings = if offline_registries {
        offline()
    } else {
        serde_json::json!({})
    };
    settings["suggestions"] = serde_json::json!({"showVulnerabilities": false});
    let config = Settings::from_value(settings)
        .expect("fixture settings are valid")
        .session_config();
    let mut state = VersionLensLspState::standard().with_config(config);
    state.open_document(VersionLensTextDocument {
        uri: case.uri.to_owned(),
        language_id: case.language.to_owned(),
        text: case.text.clone(),
        workspace_root: None,
    });
    let responses = case
        .registry
        .iter()
        .map(|stub| {
            let ecosystem = serde_json::from_value(serde_json::json!(stub.ecosystem))
                .unwrap_or_else(|error| panic!("{}: {error}", stub.ecosystem));
            RegistryResponseInput::new(stub.package, ecosystem, stub.body.clone())
        })
        .collect::<Vec<_>>();
    let work = state.document_work(case.uri).expect("document is open");
    state
        .session_for(case.uri)
        .resolve_document_with_responses(work.input, &responses)
        .await;
    let resolved = state.analyzed_document(case.uri).expect("document is open");
    Served {
        lenses: resolved.code_lenses,
        diagnostics: resolved.diagnostics,
    }
}

#[tokio::test]
async fn cargo_exact_pin_is_current() {
    let served = serve(&regression::cargo_exact_pin(), false).await;
    served.assert_status("latest 0.7.0");
    served.assert_no_bump();
}

#[tokio::test]
async fn python_exact_pin_is_current() {
    let served = serve(&regression::python_exact_pin(), false).await;
    served.assert_status("latest 2.34.2");
    served.assert_no_bump();
}

#[tokio::test]
async fn caret_range_offers_a_floor_bump_without_a_diagnostic() {
    let served = serve(&regression::caret_range_behind_latest(), false).await;
    served.assert_status("satisfies latest 2.5.14");
    assert_eq!(served.bump_versions(), ["2.5.14"]);
    assert_eq!(served.diagnostics, vec![]);
}

#[tokio::test]
async fn starlight_intermediate_releases_are_not_listed() {
    let served = serve(&regression::intermediate_releases(), false).await;
    served.assert_status("satisfies 0.41.11");
    assert_eq!(served.bump_versions(), ["0.41.11", "0.42.6"]);
    for skipped in regression::STARLIGHT_SKIPPED {
        assert!(
            served.titles().iter().all(|title| !title.contains(skipped)),
            "{skipped} listed: {:?}",
            served.titles()
        );
    }
}

#[tokio::test]
async fn github_script_sha_with_floating_and_exact_tags_is_current() {
    let served = serve(&regression::sha_pinned_action(), false).await;
    served.assert_status("latest v9.0.0");
    served.assert_no_bump();
}

#[tokio::test]
async fn install_action_branch_ref_is_fixed_without_a_lookup() {
    let served = serve(&regression::branch_action_ref(), true).await;
    served.assert_silent();
    served.assert_status("fixed just");
    served.assert_no_bump();
}

#[tokio::test]
async fn engines_vscode_is_skipped_without_an_error() {
    let served = serve(&regression::vscode_engine_field(), true).await;
    served.assert_silent();
}

#[tokio::test]
async fn setup_dotnet_unsupported_syntax_is_not_an_error() {
    let served = serve(&regression::unsupported_setup_dotnet(), true).await;
    served.assert_silent();
    served.assert_no_bump();
}

#[tokio::test]
async fn env_reference_resolves_the_runtime() {
    let served = serve(&regression::workflow_env_reference(), false).await;
    served.assert_silent();
    // The runtime resolved from `${{ env.NODE_VERSION }}` ("20") against the
    // node release list.
    served.assert_status("fixed 20");
    assert_eq!(served.bump_versions(), ["22.0.0"]);
}

#[tokio::test]
async fn steps_output_expression_is_skipped_silently() {
    let served = serve(&regression::step_output_expression(), true).await;
    served.assert_silent();
    served.assert_no_bump();
}

#[tokio::test]
async fn gradle_version_alias_resolves_through_its_library() {
    let served = serve(&regression::gradle_version_alias(), false).await;
    served.assert_silent();
    assert_eq!(served.bump_versions(), ["2.1.0"]);
    served.assert_bump_on_line(1);
}

#[tokio::test]
async fn gradle_plugin_alias_uses_the_plugin_marker() {
    let served = serve(&regression::gradle_plugin_alias(), false).await;
    served.assert_silent();
    assert_eq!(served.bump_versions(), ["2.0.0"]);
    served.assert_bump_on_line(1);
}

#[tokio::test]
async fn cargo_bare_version_is_a_caret_requirement() {
    let served = serve(&regression::cargo_bare_version(), false).await;
    served.assert_status("satisfies latest 1.0.229");
    assert_eq!(served.bump_versions(), ["1.0.229"]);
}

#[tokio::test]
async fn package_version_fields_are_not_dependencies() {
    for case in regression::package_version_fields() {
        let served = serve(&case, true).await;
        assert_eq!(served.titles(), Vec::<&str>::new(), "{}", case.uri);
        assert_eq!(served.diagnostics, vec![], "{}", case.uri);
    }
}
