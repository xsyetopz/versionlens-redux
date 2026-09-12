use super::*;

mod cache;
mod dist_tags;
mod http;
mod registry_config;

#[test]
fn resolves_update_from_registry_response_body() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput::new(
            "file:///package.json".to_owned(),
            "json".to_owned(),
            package_file_fixture("update-from-registry-response-body.json"),
            None,
        ),
        &[crate::support::tests::npm_latest_response(
            "left-pad", "1.1.0",
        )],
    );

    assert_update(&output, "1.1.0");
}

#[test]
fn resolves_npm_alias_dependencies_against_target_package() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput::new(
            "file:///package.json".to_owned(),
            "json".to_owned(),
            package_file_fixture("alias-dependencies-against-target-package.json"),
            None,
        ),
        &[RegistryResponseInput::new(
            "typescript".to_owned(),
            Npm,
            r#"{"dist-tags":{"latest":"6.0.4"}}"#.to_owned(),
        )],
    );

    assert_eq!(output.suggestions[0].dependency.name, "typescript");
    assert_update(&output, "npm:typescript@6.0.4");
}

#[test]
fn resolves_ranged_npm_alias_dependencies_preserving_range_prefix() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput::new(
            "file:///package.json".to_owned(),
            "json".to_owned(),
            package_file_fixture("ranged-npm-alias-dependencies-preserving-range-prefix.json"),
            None,
        ),
        &[RegistryResponseInput::new(
            "@types/react".to_owned(),
            Npm,
            r#"{"dist-tags":{"latest":"20.0.0"}}"#.to_owned(),
        )],
    );

    assert_eq!(output.suggestions[0].dependency.name, "@types/react");
    assert_update(&output, "npm:@types/react@^20.0.0");
}

#[test]
fn resolves_unversioned_npm_alias_dependencies_against_target_package() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput::new(
            "file:///package.json".to_owned(),
            "json".to_owned(),
            package_file_fixture("unversioned-npm-alias-dependencies-against-target-package.json"),
            None,
        ),
        &[RegistryResponseInput::new(
            "types-react".to_owned(),
            Npm,
            r#"{"dist-tags":{"latest":"19.2.7"}}"#.to_owned(),
        )],
    );

    assert_eq!(output.suggestions[0].dependency.name, "types-react");
    assert_update(&output, "npm:types-react@19.2.7");
}

#[test]
fn resolves_package_yaml_npm_alias_dependencies_preserving_alias() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput::new(
            "file:///package.yaml".to_owned(),
            "yaml".to_owned(),
            package_file_fixture("package-yaml-npm-alias-dependencies-preserving-alias.yaml"),
            None,
        ),
        &[RegistryResponseInput::new(
            "typescript".to_owned(),
            Npm,
            r#"{"dist-tags":{"latest":"7.0.0"}}"#.to_owned(),
        )],
    );

    assert_eq!(output.suggestions[0].dependency.name, "typescript");
    assert_update(&output, "npm:typescript@^7.0.0");
}

#[test]
fn resolves_deno_jsr_imports_preserving_specifier_scheme() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput::new(
            "file:///deno.json".to_owned(),
            "jsonc".to_owned(),
            package_file_fixture("deno-jsr-imports-preserving-specifier-scheme.json"),
            None,
        ),
        &[RegistryResponseInput::new(
            "@std/assert".to_owned(),
            Deno,
            r#"{"latest":"1.1.0","versions":{"1.0.0":{},"1.1.0":{}}}"#.to_owned(),
        )],
    );

    assert_eq!(output.suggestions[0].dependency.name, "@std/assert");
    assert_update(&output, "jsr:@std/assert@1.1.0");
}

#[test]
fn resolves_deno_npm_imports_preserving_specifier_scheme() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput::new(
            "file:///deno.json".to_owned(),
            "jsonc".to_owned(),
            package_file_fixture("deno-npm-imports-preserving-specifier-scheme.json"),
            None,
        ),
        &[RegistryResponseInput::new(
            "chalk".to_owned(),
            Npm,
            r#"{"dist-tags":{"latest":"5.4.0"}}"#.to_owned(),
        )],
    );

    assert_eq!(output.suggestions[0].dependency.name, "chalk");
    assert_update(&output, "npm:chalk@5.4.0");
}

#[test]
fn resolves_invalid_empty_ranges_as_invalid_range_with_latest_update() {
    let session = session_without_vulnerabilities();

    let output = session.resolve_document_with_responses(
        DocumentInput::new(
            "file:///package.json".to_owned(),
            "json".to_owned(),
            package_file_fixture("invalid-empty-ranges-as-invalid-range-with-latest-update.json"),
            None,
        ),
        &[crate::support::tests::npm_latest_response(
            "left-pad", "5.0.0",
        )],
    );

    assert_eq!(output.suggestions[0].status, "invalidRange");
    assert_eq!(output.edits[0].new_text, "5.0.0");
}

struct NpmResolutionCase<'a> {
    fixture: &'a str,
    response_body: &'a str,
    expected_status: &'a str,
    expected_latest: Option<&'a str>,
    expected_titles: &'a [&'a str],
    expected_arguments: &'a [(&'a str, &'a str)],
}

fn assert_npm_resolution_choices(case: NpmResolutionCase<'_>) {
    let session = session_without_vulnerabilities();
    let input = DocumentInput::new(
        "file:///package.json".to_owned(),
        "json".to_owned(),
        package_file_fixture(case.fixture),
        None,
    );
    let output = session.resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput::new(
            "left-pad".to_owned(),
            Npm,
            case.response_body.to_owned(),
        )],
    );
    let analysis = session.analyze_document(input);

    assert_eq!(output.suggestions[0].status, case.expected_status);
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        case.expected_latest
    );
    assert!(output.edits.is_empty());
    assert_eq!(
        crate::session::resolution::tests::lens_titles(&analysis),
        case.expected_titles
    );
    assert_eq!(
        crate::support::tests::code_lens_arguments(&analysis),
        case.expected_arguments
            .iter()
            .map(|(command, version)| vec![(*command).to_owned(), (*version).to_owned()])
            .collect::<Vec<_>>()
    );
}

#[test]
fn missing_fixed_npm_registry_version_resolves_no_match_with_update_choices() {
    assert_npm_resolution_choices(NpmResolutionCase {
        fixture: "missing-fixed-npm-registry-version-resolves-no-match-with-update-choices.json",
        response_body: r#"{
              "dist-tags": { "latest": "1.0.0" },
              "versions": { "0.5.1": {}, "0.6.0": {}, "1.0.0": {} }
            }"#,
        expected_status: "noMatch",
        expected_latest: None,
        expected_titles: &[
            "⚪ no match",
            "↑  patch 0.5.1",
            "↑  minor 0.6.0",
            "↑  latest 1.0.0",
        ],
        expected_arguments: &[
            ("updatePatch", "0.5.1"),
            ("updateMinor", "0.6.0"),
            ("update", "1.0.0"),
        ],
    });
}

#[test]
fn fixed_npm_prerelease_resolves_fixed_with_prerelease_update_choice() {
    assert_npm_resolution_choices(NpmResolutionCase {
        fixture: "fixed-npm-prerelease-resolves-fixed-with-prerelease-update-choice.json",
        response_body: r#"{"versions":{"1.0.0-beta.1":{},"1.0.0-beta.2":{},"1.0.0-beta.3":{}}}"#,
        expected_status: "fixed",
        expected_latest: Some("1.0.0-beta.1"),
        expected_titles: &["🟡 fixed 1.0.0-beta.1", "↑  beta 1.0.0-beta.3"],
        expected_arguments: &[("update", "1.0.0-beta.3")],
    });
}

#[test]
fn fixed_npm_release_resolves_fixed_with_release_update_choices() {
    assert_npm_resolution_choices(NpmResolutionCase {
        fixture: "fixed-npm-release-resolves-fixed-with-release-update-choices.json",
        response_body: r#"{"versions":{"1.1.0":{},"1.1.1":{},"1.1.2":{},"1.2.0":{},"1.2.2":{},"2.0.0":{},"2.2.2":{}}}"#,
        expected_status: "fixed",
        expected_latest: Some("1.1.1"),
        expected_titles: &[
            "🟡 fixed 1.1.1",
            "↑  patch 1.1.2",
            "↑  minor 1.2.2",
            "↑  latest 2.2.2",
        ],
        expected_arguments: &[
            ("updatePatch", "1.1.2"),
            ("updateMinor", "1.2.2"),
            ("update", "2.2.2"),
        ],
    });
}

include!("npm/workspaces.rs");

fn package_file_fixture(name: &str) -> String {
    crate::support::tests::fixture("tests/fixtures/session/resolution/tests/npm", name)
}
use super::assert_update;
