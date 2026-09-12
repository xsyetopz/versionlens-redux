use super::super::*;

#[test]
fn ci_runtime_channels_and_numeric_constraints_keep_the_declared_selector() {
    for (action, field, context, name, ecosystem, requirement, latest, body, status) in [
        (
            "actions/setup-go",
            "go-version",
            "",
            "go",
            Go,
            "stable",
            "1.25.1",
            r#"[{"version":"go1.25.1","stable":true},{"version":"go1.24.7","stable":true}]"#,
            "current",
        ),
        (
            "denoland/setup-deno",
            "deno-version",
            "",
            "deno",
            Deno,
            "lts",
            "2.1.4",
            "v2.1.4",
            "current",
        ),
        (
            "actions/setup-python",
            "python-version",
            "",
            "python",
            Python,
            "3.12",
            "3.12.2",
            r#"[{"version":"3.12.2","stable":true},{"version":"3.13.0","stable":true}]"#,
            "satisfiesLatest",
        ),
        (
            "actions/setup-java",
            "java-version",
            "distribution: temurin, ",
            "java",
            Maven,
            "21",
            "21.0.12.1",
            r#"{"releases":["jdk-21.0.12+8","jdk-21.0.12.1+1"]}"#,
            "satisfiesLatest",
        ),
        (
            "actions/setup-dotnet",
            "dotnet-version",
            "",
            "dotnet",
            Dotnet,
            "8.0.x",
            "8.0.405",
            r#"{"releases":[{"sdk":{"version":"8.0.405"}}]}"#,
            "satisfiesLatest",
        ),
        (
            "ruby/setup-ruby",
            "ruby-version",
            "",
            "ruby",
            Ruby,
            "3.3",
            "3.3.2",
            r#"{"ruby":["3.3.2","3.4.0"]}"#,
            "satisfiesLatest",
        ),
    ] {
        let text = format!(
            "steps: [{{uses: {action}@v1, with: {{{context}{field}: '{requirement}'}}}}]\n"
        );
        let output = session_without_vulnerabilities().resolve_document_with_responses(
            DocumentInput::new(
                "file:///work/.github/workflows/runtime.yml",
                "yaml",
                text,
                None,
            ),
            &[
                RegistryResponseInput::new(action, GitHub, r#"[{"name":"v1"}]"#),
                RegistryResponseInput::new(name, ecosystem, body),
            ],
        );
        let runtime = output
            .suggestions
            .iter()
            .find(|suggestion| suggestion.dependency.name == name)
            .unwrap();
        assert_eq!(runtime.status, status, "{name}: {runtime:?}");
        assert_eq!(runtime.latest.as_deref(), Some(latest), "{name}");
        assert!(output.edits.is_empty(), "{name}: {output:?}");
    }
}

#[test]
fn unsupported_ci_runtime_variants_report_errors() {
    for (action, input, context, name, _ecosystem, requirement) in [
        (
            "actions/setup-python",
            "python-version",
            "",
            "python",
            Python,
            "pypy3.10",
        ),
        (
            "actions/setup-java",
            "java-version",
            "distribution: zulu, ",
            "java",
            Maven,
            "17",
        ),
        (
            "actions/setup-dotnet",
            "dotnet-version",
            "dotnet-quality: preview, ",
            "dotnet",
            Dotnet,
            "8.0.x",
        ),
        (
            "ruby/setup-ruby",
            "ruby-version",
            "",
            "ruby",
            Ruby,
            "jruby-9.4",
        ),
    ] {
        let text = format!(
            "steps: [{{uses: {action}@v1, with: {{{context}{input}: '{requirement}'}}}}]\n"
        );
        let output = session_without_vulnerabilities().resolve_document_with_responses(
            DocumentInput::new(
                "file:///work/.github/workflows/runtime.yml",
                "yaml",
                text,
                None,
            ),
            &[RegistryResponseInput::new(
                action,
                GitHub,
                r#"[{"name":"v1"}]"#,
            )],
        );
        let runtime = output
            .suggestions
            .iter()
            .find(|suggestion| suggestion.dependency.name == name)
            .unwrap();
        assert_eq!(runtime.status, "error", "{name}: {runtime:?}");
    }
}
