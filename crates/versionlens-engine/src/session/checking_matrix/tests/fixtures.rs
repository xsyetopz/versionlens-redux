//! Regression fixtures for reported suggestion bugs. Each test states the
//! behavior its spec scenario requires.

use versionlens_model::{DocumentInput, Ecosystem};

use super::super::{ApplyCommandRequest, VersionLensSession};
use super::golden::lens_actions;
use super::test_session;
use crate::contract::{AnalyzeDocumentOutput, ResolveDocumentOutput};
use crate::{ProviderSettings, RegistryResponseInput, RegistryUrlConfig};

struct Checked {
    resolved: ResolveDocumentOutput,
    analysis: AnalyzeDocumentOutput,
    bumps: Vec<String>,
}

async fn check(
    session: &VersionLensSession,
    uri: &str,
    language: &str,
    text: &str,
    responses: &[RegistryResponseInput],
) -> Checked {
    let input = DocumentInput::new(uri, language, text, None);
    let resolved = session
        .resolve_document_with_responses(input.clone(), responses)
        .await;
    let analysis = session.analyze_document(input.clone());
    let mut bumps = Vec::new();
    for lens in &analysis.code_lenses {
        for (command, dependency_name, selected_version) in lens_actions(lens) {
            let output = session
                .apply_command_with_selected_version(ApplyCommandRequest {
                    input: input.clone(),
                    command: Some(command),
                    dependency_name: Some(dependency_name),
                    selected_version,
                    responses,
                })
                .await;
            bumps.extend(output.edits.into_iter().map(|edit| edit.new_text));
        }
    }
    bumps.sort();
    bumps.dedup();
    Checked {
        resolved,
        analysis,
        bumps,
    }
}

/// A session whose registries refuse connections, so any lookup surfaces as
/// an error instead of reaching the network.
fn offline_session() -> VersionLensSession {
    let unreachable = |ecosystem| RegistryUrlConfig {
        ecosystem,
        url: "http://127.0.0.1:9/".to_owned(),
    };
    crate::test_support::session_with_provider_settings(
        ProviderSettings {
            registry_urls: vec![
                unreachable(Ecosystem::GitHub),
                unreachable(Ecosystem::Npm),
                unreachable(Ecosystem::Dotnet),
            ],
            ..Default::default()
        },
        false,
    )
}

fn titles(checked: &Checked) -> Vec<&str> {
    checked
        .analysis
        .code_lenses
        .iter()
        .map(|lens| lens.title.as_str())
        .collect()
}

fn assert_status(checked: &Checked, status: &str) {
    assert!(
        titles(checked)
            .iter()
            .any(|title| title.ends_with(status) && !title.starts_with('↑')),
        "expected status `{status}`, lenses: {:?}",
        titles(checked)
    );
}

fn assert_no_bump(checked: &Checked) {
    assert!(
        titles(checked).iter().all(|title| !title.starts_with('↑')),
        "bump lens offered: {:?}",
        titles(checked)
    );
    assert_eq!(
        checked.bumps,
        Vec::<String>::new(),
        "lenses: {:?}",
        titles(checked)
    );
    assert_eq!(checked.resolved.edits, vec![]);
}

/// No error, and no diagnostic other than an available update.
fn assert_silent(checked: &Checked) {
    let noise = checked
        .analysis
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code.as_deref() != Some(crate::UPDATE_AVAILABLE))
        .collect::<Vec<_>>();
    assert_eq!(noise, Vec::<&crate::DiagnosticView>::new());
    assert!(
        checked
            .resolved
            .suggestions
            .iter()
            .all(|suggestion| suggestion.status != "error"),
        "suggestions: {:?}",
        checked.resolved.suggestions
    );
}

fn npm_versions(versions: &[&str], latest: &str) -> String {
    let versions = versions
        .iter()
        .map(|version| format!(r#""{version}":{{}}"#))
        .collect::<Vec<_>>()
        .join(",");
    format!(r#"{{"versions":{{{versions}}},"dist-tags":{{"latest":"{latest}"}}}}"#)
}

#[tokio::test]
async fn cargo_exact_pin_at_latest_is_current() {
    let checked = check(
        &test_session(),
        "file:///work/Cargo.toml",
        "toml",
        "[dependencies]\nzed_extension_api = \"=0.7.0\"\n",
        &[RegistryResponseInput::new(
            "zed_extension_api",
            Ecosystem::Cargo,
            r#"{"versions":[{"num":"0.7.0","yanked":false},{"num":"0.6.0","yanked":false}]}"#,
        )],
    )
    .await;
    assert_status(&checked, "latest 0.7.0");
    assert_no_bump(&checked);
}

#[tokio::test]
async fn python_exact_pin_at_latest_is_current() {
    let checked = check(
        &test_session(),
        "file:///work/requirements.txt",
        "pip-requirements",
        "requests==2.34.2\n",
        &[RegistryResponseInput::new(
            "requests",
            Ecosystem::Python,
            r#"{"info":{"version":"2.34.2"},"releases":{"2.34.1":[{"yanked":false}],"2.34.2":[{"yanked":false}]}}"#,
        )],
    )
    .await;
    assert_status(&checked, "latest 2.34.2");
    assert_no_bump(&checked);
}

#[tokio::test]
async fn caret_range_behind_latest_offers_a_floor_bump_without_a_diagnostic() {
    let checked = check(
        &test_session(),
        "file:///work/package.json",
        "json",
        "{\"devDependencies\":{\"@biomejs/biome\":\"^2.5.8\"}}\n",
        &[RegistryResponseInput::new(
            "@biomejs/biome",
            Ecosystem::Npm,
            npm_versions(&["2.5.8", "2.5.9", "2.5.10", "2.5.14"], "2.5.14"),
        )],
    )
    .await;
    assert_status(&checked, "satisfies latest 2.5.14");
    assert_eq!(checked.bumps, ["^2.5.14"]);
    assert_eq!(checked.analysis.diagnostics, vec![]);
}

#[tokio::test]
async fn intermediate_releases_are_not_listed_individually() {
    let checked = check(
        &test_session(),
        "file:///work/package.json",
        "json",
        "{\"dependencies\":{\"@astrojs/starlight\":\"^0.41.7\"}}\n",
        &[RegistryResponseInput::new(
            "@astrojs/starlight",
            Ecosystem::Npm,
            npm_versions(
                &[
                    "0.41.7", "0.41.11", "0.42.0", "0.42.1", "0.42.2", "0.42.3", "0.42.4",
                    "0.42.5", "0.42.6",
                ],
                "0.42.6",
            ),
        )],
    )
    .await;
    assert_status(&checked, "satisfies 0.41.11");
    assert_eq!(checked.bumps, ["^0.41.11", "^0.42.6"]);
    for skipped in ["0.42.0", "0.42.1", "0.42.2", "0.42.3", "0.42.4", "0.42.5"] {
        assert!(
            titles(&checked)
                .iter()
                .all(|title| !title.contains(skipped)),
            "{skipped} listed: {:?}",
            titles(&checked)
        );
    }
}

#[tokio::test]
async fn sha_pinned_action_with_floating_and_exact_tags_is_current() {
    let sha = "3a2844b7e9c422d3c10d287c895573f7108da1b3";
    let checked = check(
        &test_session(),
        "file:///work/.github/workflows/ci.yml",
        "yaml",
        &format!("steps:\n  - uses: actions/github-script@{sha} # v9.0.0\n"),
        &[RegistryResponseInput::new(
            "actions/github-script",
            Ecosystem::GitHub,
            format!(
                r#"[{{"name":"v9.0.0","commit":{{"sha":"{sha}"}}}},{{"name":"v9","commit":{{"sha":"{sha}"}}}},{{"name":"v8.0.0","commit":{{"sha":"8888888888888888888888888888888888888888"}}}}]"#
            ),
        )],
    )
    .await;
    assert_status(&checked, "latest v9.0.0");
    assert_no_bump(&checked);
}

#[tokio::test]
async fn branch_action_ref_is_fixed_without_a_lookup() {
    let checked = check(
        &offline_session(),
        "file:///work/.github/workflows/ci.yml",
        "yaml",
        "steps:\n  - uses: taiki-e/install-action@just\n",
        &[],
    )
    .await;
    assert_silent(&checked);
    assert_eq!(checked.resolved.suggestions[0].status, "fixed");
    assert_no_bump(&checked);
}

#[tokio::test]
async fn vscode_engine_field_is_skipped_without_an_error() {
    let checked = check(
        &offline_session(),
        "file:///work/package.json",
        "json",
        "{\"engines\":{\"vscode\":\"^1.75.0\"}}\n",
        &[],
    )
    .await;
    assert_silent(&checked);
}

#[tokio::test]
async fn unsupported_setup_dotnet_syntax_is_not_supported_rather_than_an_error() {
    let checked = check(
        &offline_session(),
        "file:///work/.github/workflows/ci.yml",
        "yaml",
        "steps:\n  - uses: actions/setup-dotnet@v4\n    with:\n      dotnet-version: 8.0.100.1\n",
        &[RegistryResponseInput::new(
            "actions/setup-dotnet",
            Ecosystem::GitHub,
            r#"[{"name":"v4"}]"#,
        )],
    )
    .await;
    assert_silent(&checked);
    let dotnet = checked
        .resolved
        .suggestions
        .iter()
        .find(|suggestion| suggestion.dependency.name == "dotnet")
        .unwrap_or_else(|| panic!("dotnet not checked: {:?}", checked.resolved.suggestions));
    assert_eq!(dotnet.status, "notSupported");
}

#[tokio::test]
async fn workflow_env_reference_resolves_the_runtime() {
    let checked = check(
        &test_session(),
        "file:///work/.github/workflows/ci.yml",
        "yaml",
        "env:\n  NODE_VERSION: \"20\"\njobs:\n  build:\n    steps:\n      - uses: actions/setup-node@v4\n        with:\n          node-version: ${{ env.NODE_VERSION }}\n",
        &[
            RegistryResponseInput::new("actions/setup-node", Ecosystem::GitHub, r#"[{"name":"v4"}]"#),
            RegistryResponseInput::new(
                "node",
                Ecosystem::Npm,
                r#"[{"version":"v20.0.0"},{"version":"v22.0.0"}]"#,
            ),
        ],
    )
    .await;
    assert_silent(&checked);
    let node = checked
        .resolved
        .suggestions
        .iter()
        .find(|suggestion| suggestion.dependency.name == "node")
        .unwrap_or_else(|| panic!("node not resolved: {:?}", checked.resolved.suggestions));
    assert_eq!(node.dependency.requirement, "20");
}

#[tokio::test]
async fn unresolvable_step_output_expression_is_skipped_silently() {
    let checked = check(
        &offline_session(),
        "file:///work/.github/workflows/ci.yml",
        "yaml",
        "steps:\n  - uses: actions/setup-dotnet@v4\n    with:\n      dotnet-version: ${{ steps.read.outputs.version }}\n",
        &[RegistryResponseInput::new(
            "actions/setup-dotnet",
            Ecosystem::GitHub,
            r#"[{"name":"v4"}]"#,
        )],
    )
    .await;
    assert_silent(&checked);
    assert!(
        checked
            .resolved
            .suggestions
            .iter()
            .all(|suggestion| suggestion.dependency.name == "actions/setup-dotnet"),
        "expression was looked up: {:?}",
        checked.resolved.suggestions
    );
}

#[tokio::test]
async fn gradle_catalog_version_alias_resolves_through_its_library() {
    let checked = check(
        &test_session(),
        "file:///work/gradle/libs.versions.toml",
        "toml",
        "[versions]\nkotlin = \"2.0.0\"\n\n[libraries]\nkotlin-stdlib = { module = \"org.jetbrains.kotlin:kotlin-stdlib\", version.ref = \"kotlin\" }\n",
        &[RegistryResponseInput::new(
            "org.jetbrains.kotlin:kotlin-stdlib",
            Ecosystem::Maven,
            "<metadata><versioning><versions><version>2.0.0</version><version>2.1.0</version></versions></versioning></metadata>",
        )],
    )
    .await;
    assert_silent(&checked);
    let edit = checked
        .resolved
        .edits
        .iter()
        .find(|edit| edit.new_text == "2.1.0")
        .unwrap_or_else(|| panic!("no bump: {:?}", checked.resolved));
    assert_eq!(edit.range.start.line, 1);
}

#[tokio::test]
async fn gradle_catalog_plugin_alias_uses_the_plugin_marker() {
    let checked = check(
        &test_session(),
        "file:///work/gradle/libs.versions.toml",
        "toml",
        "[versions]\nksp = \"1.0.0\"\n\n[plugins]\nksp = { id = \"com.google.devtools.ksp\", version.ref = \"ksp\" }\n",
        &[RegistryResponseInput::new(
            "com.google.devtools.ksp:com.google.devtools.ksp.gradle.plugin",
            Ecosystem::Maven,
            "<metadata><versioning><versions><version>1.0.0</version><version>2.0.0</version></versions></versioning></metadata>",
        )],
    )
    .await;
    assert_silent(&checked);
    let edit = checked
        .resolved
        .edits
        .iter()
        .find(|edit| edit.new_text == "2.0.0")
        .unwrap_or_else(|| panic!("no bump: {:?}", checked.resolved));
    assert_eq!(edit.range.start.line, 1);
}

#[tokio::test]
async fn cargo_bare_version_is_a_caret_requirement() {
    let checked = check(
        &test_session(),
        "file:///work/Cargo.toml",
        "toml",
        "[dependencies]\nserde = \"1.0.200\"\n",
        &[RegistryResponseInput::new(
            "serde",
            Ecosystem::Cargo,
            r#"{"versions":[{"num":"1.0.229","yanked":false},{"num":"1.0.200","yanked":false}]}"#,
        )],
    )
    .await;
    assert_status(&checked, "satisfies latest 1.0.229");
    assert_eq!(checked.bumps, ["1.0.229"]);
}

#[tokio::test]
async fn package_version_fields_are_not_dependencies() {
    for (uri, language, text) in [
        (
            "file:///work/package.json",
            "json",
            "{\"name\":\"demo\",\"version\":\"0.4.4\"}\n",
        ),
        (
            "file:///work/Cargo.toml",
            "toml",
            "[package]\nname = \"demo\"\nversion = \"0.4.4\"\n",
        ),
    ] {
        let checked = check(&offline_session(), uri, language, text, &[]).await;
        assert_eq!(checked.analysis.dependencies, vec![], "{uri}");
        assert_eq!(titles(&checked), Vec::<&str>::new(), "{uri}");
        assert_eq!(checked.analysis.diagnostics, vec![], "{uri}");
    }
}
