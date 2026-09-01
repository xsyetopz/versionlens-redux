use super::*;

use crate::registry::RegistryContext;
use crate::{
    AnalyzeDocumentOutput, ProviderSettings, SessionConfig, SessionConfigInput, VersionLensSession,
};
use crate::{contract, registry};
use std::io::Result as IoResult;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::id;
use std::thread::spawn;
use std::time::SystemTime;
use std::{env::temp_dir, fs::create_dir_all};
use versionlens_model::{Dependency, DocumentInput, Ecosystem};

pub(crate) fn clone_arc<T>(value: &StdArc<T>) -> StdArc<T> {
    value.clone()
}

pub(crate) fn session_config_from_input(input: SessionConfigInput) -> SessionConfig {
    input.into()
}

pub(crate) fn system_time_now() -> SystemTime {
    SystemTime::now()
}

pub(crate) fn tcp_listener_bind(addr: &str) -> IoResult<TcpListener> {
    TcpListener::bind(addr)
}

pub(crate) fn fixture(base: &str, name: &str) -> String {
    versionlens_test_support::fixture!(base, name).expect("test fixture must be readable")
}

pub(crate) fn test_session(show_vulnerabilities: bool) -> VersionLensSession {
    session_with_provider_settings(crate::default(), show_vulnerabilities)
}

pub(crate) fn session_config(
    providers: ProviderSettings,
    show_vulnerabilities: bool,
) -> SessionConfig {
    SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers,
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    }
}

pub(crate) fn session_with_provider_settings(
    providers: ProviderSettings,
    show_vulnerabilities: bool,
) -> VersionLensSession {
    crate::version_lens_session(session_config(providers, show_vulnerabilities))
}

pub(crate) fn analyze_with_responses(
    session: &VersionLensSession,
    input: &DocumentInput,
    responses: &[contract::RegistryResponseInput],
) -> AnalyzeDocumentOutput {
    session.resolve_document_with_responses(input.clone(), responses);
    session.analyze_document(input.clone())
}

pub(crate) fn registry_case(
    input: &DocumentInput,
) -> (VersionLensSession, RegistryContext, Vec<Dependency>) {
    let session = test_session(false);
    let context = registry::RegistryContext::from_document(input);
    let dependencies = session.dependencies(input);
    (session, context, dependencies)
}

pub(crate) fn registry_context_and_dependencies(
    session: &VersionLensSession,
    input: &DocumentInput,
) -> (RegistryContext, Vec<Dependency>) {
    (
        registry::RegistryContext::from_document(input),
        session.dependencies(input),
    )
}

pub(crate) fn assert_registry_urls(
    session: &VersionLensSession,
    context: &RegistryContext,
    dependency: &Dependency,
    expected: &[&str],
) {
    assert_eq!(
        session.registry_urls_with_context(dependency, context),
        expected
            .iter()
            .map(|url| (*url).to_owned())
            .collect::<Vec<_>>()
    );
}

pub(crate) fn npm_latest_response(package: &str, latest: &str) -> contract::RegistryResponseInput {
    contract::RegistryResponseInput::new(
        package.to_owned(),
        Ecosystem::Npm,
        format!(r#"{{"dist-tags":{{"latest":"{latest}"}}}}"#),
    )
}

pub(crate) fn npm_vulnerability_range_response(
    package: &str,
    id: &str,
    summary: &str,
    fixed: &str,
    reference_url: Option<&str>,
) -> contract::RegistryResponseInput {
    let mut vulnerability = serde_json::json!({
        "id": id,
        "summary": summary,
        "affected": [{
            "package": {"name": package},
            "ranges": [{"events": [{"introduced": "0"}, {"fixed": fixed}]}]
        }]
    });
    if let Some(url) = reference_url {
        vulnerability["references"] = serde_json::json!([{ "url": url }]);
    }
    contract::RegistryResponseInput::new(
        package.to_owned(),
        Ecosystem::Npm,
        serde_json::json!({"vulns": [vulnerability]}).to_string(),
    )
}

pub(crate) struct FixtureResolutionCase<'a> {
    pub(crate) session: &'a VersionLensSession,
    pub(crate) uri: &'a str,
    pub(crate) language_id: &'a str,
    pub(crate) fixture_name: &'a str,
    pub(crate) package: &'a str,
    pub(crate) ecosystem: Ecosystem,
    pub(crate) response: &'a str,
}

pub(crate) fn resolve_fixture_with_response(
    case: FixtureResolutionCase<'_>,
) -> contract::ResolveDocumentOutput {
    case.session.resolve_document_with_responses(
        DocumentInput::new(
            case.uri.to_owned(),
            case.language_id.to_owned(),
            fixture(
                "tests/fixtures/session/resolution/tests/fixed",
                case.fixture_name,
            ),
            None,
        ),
        &[contract::RegistryResponseInput::new(
            case.package.to_owned(),
            case.ecosystem,
            case.response.to_owned(),
        )],
    )
}

pub(crate) fn with_unauthorized_server<T>(request: impl FnOnce(String) -> T) -> T {
    let listener = tcp_listener_bind("127.0.0.1:0").expect("bind test server");
    let base_url = format!("http://{}", listener.local_addr().expect("server address"));
    let server = spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept request");
        let mut buffer = [0; 1024];
        let _ = stream.read(&mut buffer).expect("read request");
        stream
            .write_all(
                b"HTTP/1.1 401 Unauthorized\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
            )
            .expect("write response");
    });
    let result = request(base_url);
    server.join().expect("server thread");
    result
}

pub(crate) fn code_lens_arguments<'a>(output: &'a AnalyzeDocumentOutput) -> Vec<Vec<&'a str>> {
    code_lens_arguments_after(output, 1)
}

pub(crate) fn all_code_lens_arguments<'a>(output: &'a AnalyzeDocumentOutput) -> Vec<Vec<&'a str>> {
    code_lens_arguments_after(output, 0)
}

fn code_lens_arguments_after<'a>(
    output: &'a AnalyzeDocumentOutput,
    skip_lenses: usize,
) -> Vec<Vec<&'a str>> {
    output
        .code_lenses
        .iter()
        .skip(skip_lenses)
        .map(|lens| {
            lens.arguments
                .iter()
                .skip(2)
                .map(|value| value.as_str())
                .collect()
        })
        .collect()
}

pub(crate) fn update_code_lens_arguments<'a>(
    output: &'a AnalyzeDocumentOutput,
) -> Vec<Vec<&'a str>> {
    output
        .code_lenses
        .iter()
        .filter_map(|lens| {
            if lens.command != "versionlens.suggestion.onUpdateDependency" {
                return None;
            }
            let command = lens.arguments.get(2)?;
            let version = lens.arguments.get(3)?;
            Some(vec![command.as_str(), version.as_str()])
        })
        .collect()
}

pub(crate) fn code_lens_arguments_for_title<'a>(
    output: &'a AnalyzeDocumentOutput,
    title: &str,
) -> Vec<Vec<&'a str>> {
    output
        .code_lenses
        .iter()
        .filter(|lens| lens.title == title)
        .map(|lens| {
            lens.arguments
                .iter()
                .skip(2)
                .map(|value| value.as_str())
                .collect()
        })
        .collect()
}

pub(crate) fn code_lens_commands<'a>(output: &'a AnalyzeDocumentOutput) -> Vec<&'a str> {
    output
        .code_lenses
        .iter()
        .filter_map(|lens| lens.arguments.get(2).map(|value| value.as_str()))
        .collect()
}

pub(crate) fn assert_suggestion(
    output: &contract::ResolveDocumentOutput,
    index: usize,
    status: &str,
    latest: Option<&str>,
) {
    assert_eq!(output.suggestions[index].status, status);
    assert_eq!(output.suggestions[index].latest.as_deref(), latest);
}

pub(crate) fn assert_suggestion_without_edits(
    output: &contract::ResolveDocumentOutput,
    index: usize,
    status: &str,
    latest: Option<&str>,
) {
    assert_suggestion(output, index, status, latest);
    assert_no_edits(output);
}

pub(crate) fn assert_no_edits(output: &contract::ResolveDocumentOutput) {
    assert!(output.edits.is_empty());
}

pub(crate) fn assert_all_fixed_without_edits(output: &contract::ResolveDocumentOutput) {
    assert!(
        output
            .suggestions
            .iter()
            .all(|suggestion| suggestion.status == "fixed")
    );
    assert_no_edits(output);
}

pub(crate) fn assert_fixed_git_repository(output: &contract::ResolveDocumentOutput) {
    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("git repository")
    );
    assert_no_edits(output);
}

pub(crate) fn assert_fixed_suggestion(output: &contract::ResolveDocumentOutput, latest: &str) {
    assert_suggestion(output, 0, "fixed", Some(latest));
    assert_no_edits(output);
}

pub(crate) fn local_test_root(name: &str) -> PathBuf {
    let root = temp_dir().join(format!(
        "versionlens-{name}-{}-{}",
        id(),
        system_time_now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    create_dir_all(&root).unwrap();
    root
}

pub(crate) fn file_uri(path: &std::path::Path) -> String {
    format!("file://{}", path.to_string_lossy())
}
