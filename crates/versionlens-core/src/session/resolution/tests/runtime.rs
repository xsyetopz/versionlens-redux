use super::*;

mod constraints;
mod package_managers;
mod requests;

struct CiRuntimeCase<'a> {
    action: &'a str,
    field: &'a str,
    context: &'a str,
    name: &'a str,
    ecosystem: Ecosystem,
    requirement: &'a str,
    body: &'a str,
}

async fn resolve_ci_runtime(
    case: CiRuntimeCase<'_>,
) -> (VersionLensSession, DocumentInput, ResolveDocumentOutput) {
    let CiRuntimeCase {
        action,
        field,
        context,
        name,
        ecosystem,
        requirement,
        body,
    } = case;
    let text =
        format!("steps: [{{uses: {action}@v1, with: {{{context}{field}: '{requirement}'}}}}]\n");
    let input = DocumentInput::new(
        "file:///work/.github/workflows/runtime.yml",
        "yaml",
        text,
        None,
    );
    let responses = [
        RegistryResponseInput::new(action, GitHub, r#"[{"name":"v1"}]"#),
        RegistryResponseInput::new(name, ecosystem, body),
    ];
    let session = session_without_vulnerabilities();
    let output = session
        .resolve_document_with_responses(input.clone(), &responses)
        .await;
    (session, input, output)
}

fn runtime_suggestion<'a>(
    output: &'a ResolveDocumentOutput,
    name: &str,
) -> &'a versionlens_vscode_model::SuggestionPayload {
    output
        .suggestions
        .iter()
        .find(|suggestion| suggestion.dependency.name == name)
        .unwrap_or_else(|| panic!("missing resolved {name}: {output:?}"))
}

#[tokio::test]
async fn escaped_runtime_versions_use_the_full_encoded_edit_range() {
    for (source, name, ecosystem, body, encoded, replacement) in [
        (
            r#"steps: [{uses: "dtolnay/rust-toolchain\u00401.\u0038\u0030.0"}]"#,
            "rust",
            Cargo,
            r#"[{"name":"1.80.0"},{"name":"1.90.0"}]"#,
            r#"1.\u0038\u0030.0"#,
            "1.90.0",
        ),
        (
            r#"steps: [{uses: actions/setup-node@v4, with: {node-version: "\u0032\u0030.0.0"}}]"#,
            "node",
            Npm,
            r#"[{"version":"v20.0.0"},{"version":"v22.0.0"}]"#,
            r#"\u0032\u0030.0.0"#,
            "22.0.0",
        ),
    ] {
        let output = session_without_vulnerabilities()
            .resolve_document_with_responses(
                DocumentInput::new(
                    "file:///work/.github/workflows/ci.yml",
                    "yaml",
                    source,
                    None,
                ),
                &[
                    RegistryResponseInput::new(name, ecosystem, body),
                    RegistryResponseInput::new("actions/setup-node", GitHub, r#"[{"name":"v4"}]"#),
                ],
            )
            .await;
        assert_eq!(output.edits.len(), 1, "{output:?}");
        let edit = &output.edits[0];
        assert_eq!(
            &source[edit.range.start.character as usize..edit.range.end.character as usize],
            encoded
        );
        assert_eq!(edit.new_text, replacement);
    }
}

#[tokio::test]
async fn ci_tool_versions_are_checked_and_edited_at_their_literal_ranges() {
    let session = session_without_vulnerabilities();
    let text = "jobs: {build: {steps: [{name: '🦀', uses: actions/setup-node@v4, with: {node-version: '20.0.0'}}, {uses: oven-sh/setup-bun@v2, with: {bun-version: \"1.0.0\"}}, {uses: dtolnay/rust-toolchain@1.80.0}]}} # keep\n";
    let input = DocumentInput::new("file:///work/.github/workflows/ci.yml", "yaml", text, None);
    let responses = [
        RegistryResponseInput::new("actions/setup-node", GitHub, r#"[{"name":"v4"}]"#),
        RegistryResponseInput::new("oven-sh/setup-bun", GitHub, r#"[{"name":"v2"}]"#),
        RegistryResponseInput::new(
            "node",
            Npm,
            r#"[{"version":"v20.0.0"},{"version":"v22.0.0"}]"#,
        ),
        RegistryResponseInput::new(
            "bun",
            Npm,
            r#"[{"name":"bun-v1.0.0"},{"name":"bun-v1.2.0"}]"#,
        ),
        RegistryResponseInput::new("rust", Cargo, r#"[{"name":"1.80.0"},{"name":"1.90.0"}]"#),
    ];
    let first = session
        .resolve_document_with_responses(input.clone(), &responses)
        .await;
    assert_eq!(
        first
            .edits
            .iter()
            .map(|edit| edit.new_text.as_str())
            .collect::<Vec<_>>(),
        ["22.0.0", "1.2.0", "1.90.0"]
    );
    let start = text.find("20.0.0").unwrap();
    assert!(!first.edits.is_empty(), "{first:?}");
    assert_eq!(
        first.edits[0].range.start.character as usize,
        text[..start].encode_utf16().count()
    );
    assert_eq!(
        first.edits[0].range.end.character - first.edits[0].range.start.character,
        6
    );
    assert!(session.document_is_fresh(&input));
    let mut release = input;
    release.uri = "file:///work/.github/workflows/release.yml".to_owned();
    let second = session
        .resolve_document_with_responses(release, &responses)
        .await;
    assert_eq!(first.edits, second.edits);
}

#[tokio::test]
async fn current_bun_pin_has_one_status_lens_and_no_update() {
    let (session, input, output) = resolve_ci_runtime(CiRuntimeCase {
        action: "oven-sh/setup-bun",
        field: "bun-version",
        context: "",
        name: "bun",
        ecosystem: Npm,
        requirement: "1.4.2",
        body: r#"[{"name":"bun-v1.4.2"}]"#,
    })
    .await;
    let bun = runtime_suggestion(&output, "bun");
    assert_eq!(bun.status, "current");
    assert!(output.edits.is_empty());

    let lenses = session.analyze_document(input).code_lenses;
    assert_eq!(
        lenses
            .iter()
            .filter(|lens| lens.title.ends_with("latest 1.4.2"))
            .count(),
        1
    );
    assert!(
        lenses
            .iter()
            .filter(|lens| lens.title.ends_with("latest 1.4.2"))
            .all(|lens| lens.command.is_empty()),
        "{lenses:?}"
    );
}

#[tokio::test]
async fn compatible_bun_selector_is_not_suppressed_as_an_exact_pin() {
    let (session, input, output) = resolve_ci_runtime(CiRuntimeCase {
        action: "oven-sh/setup-bun",
        field: "bun-version",
        context: "",
        name: "bun",
        ecosystem: Npm,
        requirement: "1.4",
        body: r#"[{"name":"bun-v1.4.2"}]"#,
    })
    .await;
    let bun = runtime_suggestion(&output, "bun");
    assert_eq!(bun.status, "updateAvailable");
    assert_eq!(bun.latest.as_deref(), Some("1.4.2"));
    assert_eq!(output.edits.len(), 1, "{output:?}");
    assert_eq!(output.edits[0].new_text, "1.4.2");
    let lenses = session.analyze_document(input).code_lenses;
    let runtime_lenses = lenses
        .iter()
        .filter(|lens| lens.title.ends_with("latest 1.4.2"))
        .collect::<Vec<_>>();
    assert_eq!(runtime_lenses.len(), 1, "{lenses:?}");
    assert!(runtime_lenses.iter().all(|lens| !lens.command.is_empty()));
}

#[tokio::test]
async fn exact_node_and_java_pins_have_current_commandless_runtime_lenses() {
    for (action, field, context, name, ecosystem, version, body) in [
        (
            "actions/setup-node",
            "node-version",
            "",
            "node",
            Npm,
            "22.0.0",
            r#"[{"version":"v22.0.0"}]"#,
        ),
        (
            "actions/setup-java",
            "java-version",
            "distribution: temurin, ",
            "java",
            Maven,
            "21.0.12.1",
            r#"{"releases":["jdk-21.0.12.1+1"]}"#,
        ),
    ] {
        let (session, input, output) = resolve_ci_runtime(CiRuntimeCase {
            action,
            field,
            context,
            name,
            ecosystem,
            requirement: version,
            body,
        })
        .await;
        let runtime = runtime_suggestion(&output, name);
        assert_eq!(runtime.status, "current", "{name}: {runtime:?}");
        assert_eq!(runtime.latest.as_deref(), Some(version), "{name}");
        assert!(output.edits.is_empty(), "{name}: {output:?}");
        let lenses = session.analyze_document(input).code_lenses;
        let runtime_lenses = lenses
            .iter()
            .filter(|lens| lens.title.ends_with(&format!("latest {version}")))
            .collect::<Vec<_>>();
        assert_eq!(runtime_lenses.len(), 1, "{name}: {lenses:?}");
        assert!(runtime_lenses[0].command.is_empty(), "{name}: {lenses:?}");
    }
}

#[tokio::test]
async fn ci_runtime_pins_resolve_from_their_upstream_release_shapes_and_replay() {
    for (action, field, context, name, ecosystem, current, latest, body) in [
        (
            "actions/setup-go",
            "go-version",
            "",
            "go",
            Go,
            "1.24.0",
            "1.25.1",
            r#"[{"version":"go1.24.0","stable":true},{"version":"go1.25.1","stable":true}]"#,
        ),
        (
            "denoland/setup-deno",
            "deno-version",
            "",
            "deno",
            Deno,
            "2.0.0",
            "2.1.0",
            r#"{"cli":["v2.0.0","v2.1.0"]}"#,
        ),
        (
            "actions/setup-python",
            "python-version",
            "",
            "python",
            Python,
            "3.12.0",
            "3.12.2",
            r#"[{"version":"3.12.0","stable":true},{"version":"3.12.2","stable":true}]"#,
        ),
        (
            "actions/setup-java",
            "java-version",
            "distribution: temurin, ",
            "java",
            Maven,
            "17.0.1",
            "17.0.2",
            r#"{"releases":["jdk-17.0.1+12","jdk-17.0.2+9"]}"#,
        ),
        (
            "actions/setup-dotnet",
            "dotnet-version",
            "",
            "dotnet",
            Dotnet,
            "8.0.100",
            "8.0.200",
            r#"{"releases":[{"sdk":{"version":"8.0.100"},"sdks":[{"version":"8.0.200"}]}]}"#,
        ),
        (
            "ruby/setup-ruby",
            "ruby-version",
            "",
            "ruby",
            Ruby,
            "3.3.0",
            "3.3.2",
            r#"{"ruby":["3.3.0","3.3.2"],"jruby":["9.4.0.0"]}"#,
        ),
    ] {
        let (session, input, checked) = resolve_ci_runtime(CiRuntimeCase {
            action,
            field,
            context,
            name,
            ecosystem,
            requirement: current,
            body,
        })
        .await;
        assert_eq!(checked.edits.len(), 1, "{name}: {checked:?}");
        assert_eq!(checked.edits[0].new_text, latest, "{name}");
        let replayed = session.resolve_document(input).await;
        assert_eq!(replayed.suggestions, checked.suggestions, "{name}");
        assert_eq!(replayed.edits, checked.edits, "{name}");
    }
}

#[tokio::test]
async fn checked_runtime_constraints_preserve_their_declared_minimums() {
    let session = session_without_vulnerabilities();
    for (uri, language, text, name, ecosystem, body) in [
        (
            "file:///work/package.json",
            "json",
            r#"{"engines":{"node":">=20"}}"#,
            "node",
            Npm,
            r#"[{"version":"v20.0.0"},{"version":"v22.0.0"}]"#,
        ),
        (
            "file:///work/Cargo.toml",
            "toml",
            "[package]\nname = \"example\"\nrust-version = \"1.9\"\n",
            "rust",
            Cargo,
            r#"[{"name":"1.9.0"},{"name":"1.80.0"}]"#,
        ),
        (
            "file:///work/package.json",
            "json",
            r#"{"engines":{"npm":"next"}}"#,
            "npm",
            Npm,
            r#"{"dist-tags":{"next":"11.0.0-beta.1"},"versions":{"11.0.0-beta.1":{},"99.0.0":{}}}"#,
        ),
        (
            "file:///work/package.json",
            "json",
            r#"{"devEngines":{"packageManager":{"name":"npm","version":"^10.0.0"}}}"#,
            "npm",
            Npm,
            r#"{"versions":{"10.0.0":{},"10.9.0":{},"11.0.0":{}}}"#,
        ),
    ] {
        let input = DocumentInput::new(uri, language, text, None);
        let responses = [RegistryResponseInput::new(name, ecosystem, body)];
        let output = session
            .resolve_document_with_responses(input.clone(), &responses)
            .await;
        assert!(output.edits.is_empty());
        assert_eq!(output.suggestions[0].status, "satisfiesLatest");
        assert!(
            session
                .analyze_document(input.clone())
                .code_lenses
                .iter()
                .all(|lens| lens.command.is_empty())
        );
        let command = session
            .apply_command_with_selected_version(crate::ApplyCommandRequest {
                input,
                command: Some("update"),
                dependency_name: Some(name),
                selected_version: Some("99.0.0"),
                responses: &responses,
            })
            .await;
        assert!(command.edits.is_empty());
    }
}

#[tokio::test]
async fn runtime_only_documents_resolve_cache_and_edit_pins() {
    for (file, text, name, ecosystem, body, selected) in [
        (
            ".nvmrc",
            "v20.0.0 # supported\n",
            "node",
            Npm,
            r#"[{"version":"v20.0.0"},{"version":"v22.0.0"}]"#,
            "v22.0.0",
        ),
        (
            ".node-version",
            "20.0.0\n",
            "node",
            Npm,
            r#"[{"version":"v20.0.0"},{"version":"v22.0.0"}]"#,
            "22.0.0",
        ),
        (
            ".bun-version",
            "1.0.0\n",
            "bun",
            Npm,
            r#"[{"name":"bun-v1.0.0"},{"name":"bun-v1.2.0"}]"#,
            "1.2.0",
        ),
        (
            "rust-toolchain",
            "1.80.0\n",
            "rust",
            Cargo,
            r#"[{"name":"1.80.0"},{"name":"1.90.0"}]"#,
            "1.90.0",
        ),
        (
            "rust-toolchain.toml",
            "[toolchain]\nchannel = '1.80.0' # keep\n",
            "rust",
            Cargo,
            r#"[{"name":"1.80.0"},{"name":"1.90.0"}]"#,
            "1.90.0",
        ),
    ] {
        let session = session_without_vulnerabilities();
        let provider = if ecosystem == Cargo { "cargo" } else { "npm" };
        let mut config = session.config.clone();
        config.enabled_providers =
            vec![crate::enabled_provider_config_from_name(provider).unwrap()];
        let session = crate::version_lens_session(config.clone());
        let input = DocumentInput::new(format!("file:///work/{file}"), "plaintext", text, None);
        let responses = [RegistryResponseInput::new(name, ecosystem, body)];
        let checked = session
            .resolve_document_with_responses(input.clone(), &responses)
            .await;
        assert_eq!(checked.suggestions.len(), 1, "{file}: {checked:?}");
        assert_eq!(checked.suggestions[0].status, "updateAvailable", "{file}");
        assert_eq!(checked.edits.len(), 1, "{file}");
        assert_eq!(checked.edits[0].new_text, selected);
        assert!(session.document_is_fresh(&input));
        let analysis = session.analyze_document(input.clone());
        assert!(analysis.is_supported_manifest);
        assert!(!analysis.code_lenses.is_empty());
        let command = session
            .apply_command_with_selected_version(crate::ApplyCommandRequest {
                input: input.clone(),
                command: Some("update"),
                dependency_name: Some(name),
                selected_version: None,
                responses: &responses,
            })
            .await;
        assert_eq!(command.edits, checked.edits);
        config.enabled_providers =
            vec![crate::enabled_provider_config_from_name("composer").unwrap()];
        let disabled = crate::version_lens_session(config).analyze_document(input);
        assert!(!disabled.is_supported_manifest);
        assert!(disabled.dependencies.is_empty());
    }
}

#[tokio::test]
async fn floating_rust_channels_are_verified_without_changing_the_channel() {
    let session = session_without_vulnerabilities();
    let output = session.resolve_document_with_responses(
        DocumentInput::new("file:///work/.github/workflows/ci.yml", "yaml", "steps:\n  - uses: dtolnay/rust-toolchain@nightly\n", None),
        &[RegistryResponseInput::new("rust", Cargo, "date = \"2026-09-09\"\n[pkg.rust]\nversion = \"1.100.0-nightly (4aa1fbcf4 2026-09-08)\"\n")],
    ).await;
    assert_eq!(output.suggestions[0].status, "current");
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("1.100.0-nightly")
    );
    assert!(output.edits.is_empty());
}

#[tokio::test]
async fn unresolved_runtime_expressions_have_explicit_cached_failures() {
    let session = session_without_vulnerabilities();
    let input = DocumentInput::new(
        "file:///work/.github/workflows/ci.yml",
        "yaml",
        "steps:\n  - uses: actions/setup-node@v4\n    with:\n      node-version: ${{ inputs.node }}\n",
        None,
    );
    let output = session
        .resolve_document_with_responses(
            input.clone(),
            &[RegistryResponseInput::new(
                "actions/setup-node",
                GitHub,
                r#"[{"name":"v4"}]"#,
            )],
        )
        .await;
    let node = output
        .suggestions
        .iter()
        .find(|suggestion| suggestion.dependency.name == "node")
        .unwrap();
    assert_eq!(node.status, "error");
    assert!(
        node.latest
            .as_deref()
            .unwrap()
            .contains("literal value or a finite matrix")
    );
    assert!(session.document_is_fresh(&input));
}
