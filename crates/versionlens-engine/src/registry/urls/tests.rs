use std::env::temp_dir;
use std::fs::create_dir_all;
use std::fs::remove_dir_all;
use std::fs::write;
use std::process::id;
use versionlens_model::DocumentInput;

use crate::{ProviderSettings, RegistryUrlConfig, SessionConfig};
use versionlens_model::Ecosystem::*;

const GRADLE_PLUGIN_MARKER_URLS: &[&str] = &[
    "https://plugins.gradle.org/m2/com/github/ben-manes/versions/com.github.ben-manes.versions.gradle.plugin/maven-metadata.xml",
    "https://repo.maven.apache.org/maven2/com/github/ben-manes/versions/com.github.ben-manes.versions.gradle.plugin/maven-metadata.xml",
];

include!("tests/hosted.rs");
include!("tests/gradle/catalogs.rs");
include!("tests/gradle/scripts.rs");
include!("tests/jvm.rs");
include!("tests/configured.rs");
include!("tests/dotnet.rs");
fn package_file_fixture(name: &str) -> String {
    crate::support::tests::fixture("tests/fixtures/core-scenarios/registry/urls/tests", name)
}

fn registry_input(uri: &str, language: &str, fixture: &str) -> DocumentInput {
    DocumentInput::new(
        uri.to_owned(),
        language.to_owned(),
        package_file_fixture(fixture),
        None,
    )
}

fn registry_context_and_dependencies(
    session: &crate::VersionLensSession,
    input: &DocumentInput,
) -> (
    crate::registry::RegistryContext,
    Vec<versionlens_model::Dependency>,
) {
    crate::support::tests::registry_context_and_dependencies(session, input)
}

fn assert_first_registry_urls(
    session: &crate::VersionLensSession,
    input: &DocumentInput,
    expected: &[&str],
) {
    let (context, dependencies) = registry_context_and_dependencies(session, input);
    crate::support::tests::assert_registry_urls(session, &context, &dependencies[0], expected);
}

fn assert_registry_urls_for_fixture(
    uri: &str,
    language: &str,
    fixture: &str,
    dependency_name: &str,
    expected_urls: &[&str],
) {
    let session = crate::support::tests::test_session(false);
    let input = DocumentInput::new(
        uri.to_owned(),
        language.to_owned(),
        package_file_fixture(fixture),
        None,
    );
    let (context, dependencies) = registry_context_and_dependencies(&session, &input);

    assert_eq!(dependencies[0].name, dependency_name);
    crate::support::tests::assert_registry_urls(
        &session,
        &context,
        &dependencies[0],
        expected_urls,
    );
}

struct WorkspaceRegistryCase<'a> {
    root_name: &'a str,
    settings: &'a str,
    relative_document: &'a str,
    language: &'a str,
    fixture: &'a str,
    dependency_name: &'a str,
    expected_urls: &'a [&'a str],
}

fn assert_workspace_registry_urls(case: WorkspaceRegistryCase<'_>) {
    let root = temp_dir().join(format!("versionlens-{}-{}", case.root_name, id()));
    let document = root.join(case.relative_document);
    if let Some(parent) = document.parent() {
        create_dir_all(parent).unwrap();
    }
    write(root.join("settings.gradle"), case.settings).unwrap();

    let session = crate::support::tests::test_session(false);
    let input = DocumentInput::new(
        format!("file://{}", document.display()),
        case.language.to_owned(),
        package_file_fixture(case.fixture),
        Some(root.to_string_lossy().into_owned()),
    );
    let (context, dependencies) = registry_context_and_dependencies(&session, &input);

    assert_eq!(dependencies[0].name, case.dependency_name);
    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        case.expected_urls
            .iter()
            .map(|url| (*url).to_owned())
            .collect::<Vec<_>>()
    );

    remove_dir_all(root).unwrap();
}

impl crate::VersionLensSession {
    pub(crate) fn registry_urls(&self, dependency: &versionlens_model::Dependency) -> Vec<String> {
        self.registry_urls_with_context(dependency, &crate::default())
    }

    pub(crate) fn registry_urls_with_context(
        &self,
        dependency: &versionlens_model::Dependency,
        context: &crate::registry::RegistryContext,
    ) -> Vec<String> {
        self.registry_endpoints_with_context(dependency, context)
            .into_iter()
            .map(|endpoint| endpoint.url)
            .collect()
    }
}
