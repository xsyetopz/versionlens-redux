use versionlens_model::Ecosystem;
use versionlens_model::Ecosystem::*;
use versionlens_model::{Dependency, DocumentInput};
use versionlens_parsers::parse_document;

use crate::RegistryResponseInput;
use crate::contract::ResolveDocumentOutput;
use crate::registry::RegistryContext;

use crate::{ProviderSettings, RegistryUrlConfig, VersionLensSession};
use std::env::temp_dir;
use std::fs::{create_dir_all, remove_dir_all, write};
use std::process::id;

fn standard_session() -> VersionLensSession {
    crate::support::tests::test_session(true)
}

fn session_without_vulnerabilities() -> VersionLensSession {
    crate::support::tests::test_session(false)
}

mod dotnet;
mod error;
mod fixed;
mod github;
mod go;
mod hex;
mod maven;
mod npm;
mod project;

fn lens_titles(output: &crate::AnalyzeDocumentOutput) -> Vec<&str> {
    output
        .code_lenses
        .iter()
        .map(|lens| lens.title.as_str())
        .collect()
}

fn lens_titles_and_arguments(output: &crate::AnalyzeDocumentOutput) -> (Vec<&str>, Vec<Vec<&str>>) {
    (
        lens_titles(output),
        crate::support::tests::code_lens_arguments(output),
    )
}

fn registry_case(input: &DocumentInput) -> (VersionLensSession, RegistryContext, Vec<Dependency>) {
    crate::support::tests::registry_case(input)
}

pub(super) fn registry_case_with_expected_urls(
    input: &DocumentInput,
    expected_urls: &[&[&str]],
) -> (VersionLensSession, RegistryContext, Vec<Dependency>) {
    let (session, context, dependencies) = registry_case(input);
    for (dependency, expected) in dependencies.iter().zip(expected_urls) {
        assert_eq!(
            session.registry_urls_with_context(dependency, &context),
            expected
                .iter()
                .map(|url| (*url).to_owned())
                .collect::<Vec<_>>()
        );
    }
    (session, context, dependencies)
}

pub(super) fn assert_single_auth_header(
    context: &RegistryContext,
    ecosystem: Ecosystem,
    url: &str,
    value: &str,
) {
    let headers = context.auth_headers_for_url(ecosystem, url);
    assert_eq!(headers.len(), 1);
    assert_eq!(headers[0].name, "authorization");
    assert_eq!(headers[0].value, value);
}

pub(super) fn assert_dependency_registry_url(
    session: &VersionLensSession,
    dependencies: &[Dependency],
    index: usize,
    context: &RegistryContext,
    expected: &[&str],
) {
    assert_eq!(
        session.registry_urls_with_context(&dependencies[index], context),
        expected
            .iter()
            .map(|url| (*url).to_owned())
            .collect::<Vec<_>>()
    );
}

pub(super) fn assert_fixed_git_repository_suggestion(output: &ResolveDocumentOutput) {
    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("git repository")
    );
}

pub(super) fn assert_update(output: &ResolveDocumentOutput, expected: &str) {
    assert_eq!(output.suggestions[0].status, "updateAvailable");
    assert_eq!(output.edits[0].new_text, expected);
}
