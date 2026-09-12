use super::super::*;

#[test]
fn runtime_release_requests_use_configured_source_providers() {
    for (uri, language, text, ecosystem, body, path, expected) in [
        (
            "file:///work/package.json",
            "json",
            r#"{"packageManager":"pnpm@9.0.0"}"#,
            Npm,
            r#"{"dist-tags":{"latest":"10.0.0"},"versions":{"9.0.0":{},"10.0.0":{}}}"#,
            "/repos/pnpm",
            "10.0.0",
        ),
        (
            "file:///work/.github/workflows/ci.yml",
            "yaml",
            "steps:\n  - uses: dtolnay/rust-toolchain@1.80.0\n",
            GitHub,
            r#"[{"name":"1.80.0"},{"name":"1.90.0"}]"#,
            "/repos/rust-lang/rust/tags",
            "1.90.0",
        ),
    ] {
        let (url, server) = super::super::github::github_api_server(vec![(200, body)]);
        let session = crate::support::tests::session_with_provider_settings(
            ProviderSettings {
                registry_urls: vec![RegistryUrlConfig { ecosystem, url }],
                ..crate::default()
            },
            false,
        );
        let output = session.resolve_document(DocumentInput::new(uri, language, text, None));
        assert_eq!(server.join().unwrap(), [path]);
        assert_eq!(output.edits.len(), 1, "{output:?}");
        assert_eq!(output.edits[0].new_text, expected);
    }
}
