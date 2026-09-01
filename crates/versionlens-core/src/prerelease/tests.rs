use crate::RegistryResponseInput;
use crate::{PrereleaseTagConfig, ProviderSettings, SessionConfig};
use versionlens_model::DocumentInput;

use versionlens_model::Ecosystem::*;

fn assert_prerelease_update_arguments(output: &crate::AnalyzeDocumentOutput) {
    assert_eq!(
        crate::support::tests::update_code_lens_arguments(output),
        [vec!["update", "6.0.3"], vec!["update", "7.0.0-beta.1"]]
    );
}

#[test]
fn show_prereleases_allows_prerelease_updates() {
    let session = prerelease_session();

    let input = DocumentInput::new(
        "file:///Directory.Packages.props".to_owned(),
        "xml".to_owned(),
        package_file_fixture("prereleases-allows-prerelease-updates.Packages.props"),
        None,
    );
    let output = session.resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput::new(
            "Newtonsoft.Json".to_owned(),
            Dotnet,
            r#"{"versions":["13.0.3","14.0.0-beta.1"]}"#.to_owned(),
        )],
    );
    let analysis = session.analyze_document(input);
    let update_arguments =
        crate::support::tests::code_lens_arguments_for_title(&analysis, "↑  beta 14.0.0-beta.1");

    assert_eq!(output.suggestions[0].status, "noMatch");
    assert!(output.edits.is_empty());
    assert_eq!(update_arguments, [vec!["update", "14.0.0-beta.1"]]);
}

#[test]
fn show_prereleases_applies_to_composer_update_choices() {
    let session = prerelease_session();

    let input = DocumentInput::new(
        "file:///repo/composer.json".to_owned(),
        "json".to_owned(),
        package_file_fixture("prereleases-applies-to-composer-update-choices.json"),
        None,
    );

    session.resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput::new(
            "php-parallel-lint/php-parallel-lint".to_owned(),
            Composer,
            r#"{
              "packages": {
                "php-parallel-lint/php-parallel-lint": [
                  { "version": "v3.1.3" },
                  { "version": "v3.2.0-beta.1" }
                ]
              }
            }"#
            .to_owned(),
        )],
    );

    let analysis = session.analyze_document(input);

    assert!(
        analysis
            .code_lenses
            .iter()
            .any(|lens| lens.title == "↑  beta 3.2.0-beta.1")
    );
}

#[test]
fn show_prereleases_applies_to_npm_versions() {
    let session = prerelease_session();

    let input = DocumentInput::new(
        "file:///package.json".to_owned(),
        "json".to_owned(),
        package_file_fixture("prereleases-applies-to-npm-versions.json"),
        None,
    );
    let responses = [RegistryResponseInput::new(
        "typescript".to_owned(),
        Npm,
        r#"{"dist-tags":{"latest":"6.0.3"},"versions":{"6.0.3":{},"7.0.0-beta.1":{}}}"#.to_owned(),
    )];

    session.resolve_document_with_responses(input.clone(), &responses);
    let output = session.analyze_document(input);
    assert_prerelease_update_arguments(&output);
}

#[test]
fn show_prereleases_keeps_npm_prerelease_choice_when_fixed_version_is_latest() {
    let session = prerelease_session();

    let input = DocumentInput::new(
        "file:///package.json".to_owned(),
        "json".to_owned(),
        package_file_fixture(
            "prereleases-keeps-npm-prerelease-choice-when-fixed-version-is-latest.json",
        ),
        None,
    );
    let responses = [RegistryResponseInput::new(
        "left-pad".to_owned(),
        Npm,
        r#"{
          "dist-tags": { "latest": "3.0.0" },
          "versions": {
            "1.0.0": {},
            "1.1.0-alpha.1": {},
            "2.0.0": {},
            "2.1.0": {},
            "3.0.0": {},
            "4.0.0-next": {}
          }
        }"#
        .to_owned(),
    )];

    let resolved = session.resolve_document_with_responses(input.clone(), &responses);
    let output = session.analyze_document(input);
    let titles = output
        .code_lenses
        .iter()
        .map(|lens| lens.title.as_str())
        .collect::<Vec<_>>();
    let arguments = crate::support::tests::code_lens_arguments(&output);

    assert_eq!(resolved.suggestions[0].status, "current");
    assert_eq!(resolved.suggestions[0].latest.as_deref(), Some("3.0.0"));
    assert!(resolved.edits.is_empty());
    assert_eq!(
        titles,
        [
            "🟢 latest 3.0.0",
            "↓  downgrade 2.1.0",
            "↑  next 4.0.0-next"
        ]
    );
    assert_eq!(
        arguments,
        [vec!["update", "2.1.0"], vec!["update", "4.0.0-next"]]
    );
}

#[test]
fn prerelease_tag_filters_apply_to_responses() {
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: ProviderSettings {
            prerelease_tags: vec![PrereleaseTagConfig {
                ecosystem: Npm,
                tags: vec!["beta".to_owned()],
            }],
            ..crate::default()
        },
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: true,
        http: versionlens_http::standard_http_config(),
    });

    let input = DocumentInput::new(
        "file:///package.json".to_owned(),
        "json".to_owned(),
        package_file_fixture("tag-filters-apply-to-responses.json"),
        None,
    );
    let responses = [RegistryResponseInput::new("typescript".to_owned(), Npm, r#"{"dist-tags":{"latest":"6.0.3"},"versions":{"6.0.3":{},"7.0.0-beta.1":{},"8.0.0-rc.1":{}}}"#
            .to_owned())];

    session.resolve_document_with_responses(input.clone(), &responses);
    let output = session.analyze_document(input);
    assert_prerelease_update_arguments(&output);
}

#[test]
fn prerelease_ranges_can_resolve_prerelease_versions_when_hidden() {
    let session = crate::support::tests::test_session(true);

    let output = session.resolve_document_with_responses(
        DocumentInput::new(
            "file:///package.json".to_owned(),
            "json".to_owned(),
            package_file_fixture("ranges-can-resolve-prerelease-versions-when-hidden.json"),
            None,
        ),
        &[RegistryResponseInput::new(
            "typescript".to_owned(),
            Npm,
            r#"{"versions":{"2.0.0-beta.1":{}}}"#.to_owned(),
        )],
    );

    assert_eq!(output.edits[0].new_text, "^2.0.0-beta.1");
}

#[test]
fn show_prereleases_applies_to_python_releases() {
    let session = prerelease_session();

    let output = session.resolve_document_with_responses(
        DocumentInput::new(
            "file:///requirements.txt".to_owned(),
            "pip-requirements".to_owned(),
            package_file_fixture("prereleases-applies-to-python-releases.txt"),
            None,
        ),
        &[RegistryResponseInput::new(
            "flask".to_owned(),
            Python,
            r#"{"info":{"version":"3.0.0"},"releases":{"3.0.0":[],"4.0.0rc1":[]}}"#.to_owned(),
        )],
    );

    assert_eq!(output.edits[0].new_text, "==4.0.0rc1");
}

fn prerelease_session() -> crate::VersionLensSession {
    crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: crate::default(),
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: true,
        http: versionlens_http::standard_http_config(),
    })
}

fn package_file_fixture(name: &str) -> String {
    crate::support::tests::fixture("tests/fixtures/core-scenarios/prerelease/tests", name)
}
