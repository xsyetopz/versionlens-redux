use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde::Deserialize;
use versionlens_model::{DocumentInput, Ecosystem, TextEdit, VersionableKind};

use super::{VersionLensSession, WorkspaceDiscoveryOptions};
use crate::{EnabledProviderConfig, RegistryResponseInput};

#[path = "support.rs"]
mod support;

use support::*;

#[path = "actions.rs"]
mod actions;

#[path = "tests/regressions.rs"]
mod regressions;

#[derive(Debug, Deserialize)]
struct ManifestCase {
    kind: String,
    path: String,
    language: String,
    text: String,
    #[serde(flatten)]
    dependency: ManifestDependency,
    #[serde(flatten)]
    resolution: ManifestResolution,
}

#[derive(Debug, Deserialize)]
struct ManifestDependency {
    package: String,
    response_package: Option<String>,
    ecosystem: String,
    requirement: String,
    range_text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ManifestResolution {
    latest: Option<String>,
    selected: Option<String>,
    edit: Option<String>,
    expected_text: Option<String>,
    status: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CategoryCase {
    name: String,
    path: String,
    text: String,
    package: String,
    kind: String,
    status: Option<String>,
    edit: Option<String>,
}

#[test]
fn supported_manifests_complete_the_checking_pipeline() {
    let cases = manifest_cases();
    assert_authoritative_case_coverage(&cases);
    assert_editor_selector_coverage(&cases);
    let workspace = crate::workspace::tests::support::TestWorkspace::new("checking-coverage");
    for case in &cases {
        workspace.write(&case.path, &case.text);
    }
    let session = test_session();
    let discovered = session
        .discover_workspace_documents(WorkspaceDiscoveryOptions::new(vec![
            workspace.path().to_owned(),
        ]))
        .collect::<Result<Vec<_>, _>>()
        .expect("coverage workspace discovery must have no failures");
    assert_eq!(
        discovered.len(),
        cases.len(),
        "every case must be discovered"
    );

    let by_path = cases
        .iter()
        .map(|case| (case.path.as_str(), case))
        .collect::<BTreeMap<_, _>>();
    for input in discovered {
        let relative = crate::workspace_path(&input.uri)
            .and_then(|path| path.strip_prefix(workspace.path()).ok().map(Path::to_owned))
            .expect("discovered URI must remain below its root");
        let relative = relative.to_string_lossy().replace('\\', "/");
        let case = by_path
            .get(relative.as_str())
            .unwrap_or_else(|| panic!("unexpected discovered manifest {relative}"));
        assert_manifest_pipeline(&session, &input, case);
    }
}

#[test]
fn versionable_categories_remain_distinct_and_safe() {
    let cases = category_cases();
    let actual_kinds = cases
        .iter()
        .map(|case| case.kind.clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(actual_kinds, versionable_kind_names());
    let workspace = crate::workspace::tests::support::TestWorkspace::new("checking-categories");
    fs::create_dir_all(workspace.root.join("foo")).unwrap();
    let session = test_session();
    let responses = [
        RegistryResponseInput::new("foo", Ecosystem::Npm, npm_response("foo")),
        RegistryResponseInput::new(
            "node",
            Ecosystem::Npm,
            r#"[{"version":"v20.0.0"},{"version":"v22.0.0"}]"#,
        ),
        RegistryResponseInput::new(
            "npm",
            Ecosystem::Npm,
            r#"{"versions":{"1.0.0":{},"2.0.0":{}},"dist-tags":{"latest":"2.0.0"}}"#,
        ),
    ];

    for case in cases {
        let input = DocumentInput::new(
            crate::workspace_file_uri(&workspace.write(&case.path, &case.text)).unwrap(),
            "json",
            &case.text,
            Some(crate::workspace_file_uri(workspace.path()).unwrap()),
        );
        let dependency = session
            .dependencies(&input)
            .into_iter()
            .find(|dependency| dependency.name == case.package)
            .unwrap_or_else(|| panic!("{} did not parse {}", case.name, case.package));
        assert_eq!(
            format!("{:?}", dependency.versionable_kind()),
            case.kind,
            "{}",
            case.name
        );
        let output = session.resolve_document_with_responses(input.clone(), &responses);
        if let Some(status) = case.status.as_deref() {
            let suggestion = suggestion_for(&output, &case.package);
            assert_eq!(suggestion.status, status, "{}", case.name);
        }
        match case.edit.as_deref() {
            Some(edit) => assert_eq!(
                output.edits.as_slice(),
                [TextEdit {
                    range: dependency.requirement_range,
                    new_text: edit.to_owned(),
                }],
                "{}",
                case.name
            ),
            None => assert!(
                output.edits.is_empty(),
                "{} changed its declared value: {output:?}",
                case.name
            ),
        }
        let rendered = session.analyze_document(input);
        if case.status.is_some() {
            let suggestion = suggestion_for(&output, &case.package);
            assert_semantic_target_uniqueness(&rendered, suggestion, &case.name);
        }
        assert!(
            !rendered.code_lenses.is_empty(),
            "{} did not render",
            case.name
        );
        if case.kind == "RuntimeConstraint" {
            assert!(
                rendered
                    .code_lenses
                    .iter()
                    .all(|lens| lens.command.is_empty())
            );
        }
    }
}

#[test]
fn disabled_provider_is_excluded_from_discovery_and_analysis() {
    let workspace = crate::workspace::tests::support::TestWorkspace::new("disabled-coverage");
    workspace.write("package.json", r#"{"dependencies":{"foo":"1.0.0"}}"#);
    workspace.write("Cargo.toml", "[dependencies]\nfoo = \"1.0.0\"\n");
    let mut config = crate::support::tests::session_config(crate::default(), false);
    config.enabled_providers = vec![EnabledProviderConfig {
        ecosystem: Ecosystem::Cargo,
        manifest_kind: None,
    }];
    let session = VersionLensSession::new(config);
    let discovered = session
        .discover_workspace_documents(WorkspaceDiscoveryOptions::new(vec![
            workspace.path().to_owned(),
        ]))
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(discovered.len(), 1);
    assert!(discovered[0].uri.ends_with("/Cargo.toml"));

    let npm = DocumentInput::new(
        crate::workspace_file_uri(&workspace.root.join("package.json")).unwrap(),
        "json",
        r#"{"dependencies":{"foo":"1.0.0"}}"#,
        None,
    );
    let analyzed = session.analyze_document(npm.clone());
    assert!(!analyzed.is_supported_manifest);
    assert!(analyzed.dependencies.is_empty());
    assert!(session.resolve_document(npm).suggestions.is_empty());
}

#[test]
fn response_failure_is_rendered_and_never_edited() {
    let session = test_session();
    let input = DocumentInput::new(
        "file:///coverage/package.json",
        "json",
        r#"{"dependencies":{"foo":"1.0.0"}}"#,
        None,
    );
    let response = RegistryResponseInput::new("foo", Ecosystem::Npm, r#"{"status":"E404"}"#);
    let output = session.resolve_document_with_responses(input.clone(), &[response]);
    assert_eq!(suggestion_for(&output, "foo").status, "error");
    assert!(output.edits.is_empty());
    assert!(session.document_is_fresh(&input));
    let rendered = session.analyze_document(input);
    assert_eq!(rendered.status.error_count, 1);
    assert_eq!(rendered.code_lenses.len(), 1);
}

#[test]
fn dotnet_fixed_status_survives_memory_cache() {
    let session = test_session();
    let input = DocumentInput::new(
        "file:///coverage/app.csproj",
        "xml",
        "<Project><ItemGroup><PackageReference Include=\"Foo\" Version=\"1.0.0\" /></ItemGroup></Project>\n",
        None,
    );
    let response = RegistryResponseInput::new(
        "Foo",
        Ecosystem::Dotnet,
        r#"{"versions":["1.0.0","2.0.0"]}"#,
    );
    let first = session.resolve_document_with_responses(input.clone(), &[response]);
    let cached = session.resolve_document(input);
    assert_eq!(suggestion_for(&first, "Foo").status, "fixed");
    assert_eq!(
        suggestion_for(&cached, "Foo").status,
        suggestion_for(&first, "Foo").status
    );
    assert_eq!(
        suggestion_for(&cached, "Foo").latest,
        suggestion_for(&first, "Foo").latest
    );
}

#[test]
fn successful_result_survives_memory_clear_and_session_restart() {
    let workspace = crate::workspace::tests::support::TestWorkspace::new("checking-cache");
    let directory = workspace.root.join("cache");
    let input = DocumentInput::new(
        "file:///coverage/package.json",
        "json",
        r#"{"dependencies":{"foo":"1.0.0"}}"#,
        None,
    );
    let first = test_session().with_persistent_cache(&directory).unwrap();
    assert!(!first.document_is_fresh(&input));
    let response = RegistryResponseInput::new("foo", Ecosystem::Npm, npm_response("foo"));
    let checked = first.resolve_document_with_responses(input.clone(), &[response]);
    assert_eq!(
        suggestion_for(&checked, "foo").latest.as_deref(),
        Some("2.0.0")
    );
    assert!(first.document_is_fresh(&input));
    first.clear_memory_cache();
    assert!(first.document_is_fresh(&input));

    let restarted = test_session().with_persistent_cache(&directory).unwrap();
    assert!(restarted.document_is_fresh(&input));
    let restored = restarted.resolve_document(input.clone());
    assert_eq!(
        suggestion_for(&restored, "foo").latest.as_deref(),
        Some("2.0.0")
    );
    assert_eq!(
        apply_edits(&input.text, &restored.edits),
        r#"{"dependencies":{"foo":"2.0.0"}}"#
    );
    fs::remove_dir_all(directory).unwrap();
}

fn assert_manifest_pipeline(
    session: &VersionLensSession,
    input: &DocumentInput,
    case: &ManifestCase,
) {
    let dependency = assert_manifest_input(session, input, case);
    let response = RegistryResponseInput::new(
        case.dependency
            .response_package
            .as_deref()
            .unwrap_or(&case.dependency.package),
        dependency.ecosystem,
        response_body(case),
    );
    let resolved =
        session.resolve_document_with_responses(input.clone(), std::slice::from_ref(&response));
    let suggestion = suggestion_for(&resolved, &case.dependency.package);
    assert_eq!(
        suggestion.status,
        case.resolution
            .status
            .as_deref()
            .unwrap_or("updateAvailable"),
        "{} suggestion: {suggestion:?}",
        case.kind,
    );
    assert_eq!(
        suggestion.latest.as_deref(),
        case.resolution.latest.as_deref(),
        "{} suggestion: {suggestion:?}",
        case.kind,
    );
    assert!(
        resolved
            .suggestions
            .iter()
            .all(|suggestion| suggestion.status != "error"),
        "{} produced an unchecked error: {resolved:?}",
        case.kind
    );
    assert!(
        session.document_is_fresh(input),
        "{} did not cache its result",
        case.kind
    );
    if case.resolution.selected.is_none() {
        let cached = session.resolve_document(input.clone());
        let cached_suggestion = suggestion_for(&cached, &case.dependency.package);
        assert_eq!(
            cached_suggestion.status, suggestion.status,
            "{} cache status",
            case.kind
        );
        assert_eq!(
            cached_suggestion.latest, suggestion.latest,
            "{} cache latest",
            case.kind
        );
    }
    assert_manifest_edits(ManifestEditCheck {
        session,
        input,
        case,
        dependency: &dependency,
        response: &response,
        resolved: &resolved,
    });
    let rendered = session.analyze_document(input.clone());
    assert_semantic_target_uniqueness(&rendered, suggestion, &case.kind);
    assert!(
        !rendered.code_lenses.is_empty(),
        "{} had no rendered result",
        case.kind
    );
    assert!(
        rendered.status.visible,
        "{} had no rendered status",
        case.kind
    );
}

fn assert_semantic_target_uniqueness(
    rendered: &crate::AnalyzeDocumentOutput,
    suggestion: &versionlens_vscode_model::SuggestionPayload,
    case: &str,
) {
    let targets = rendered
        .code_lenses
        .iter()
        .filter(|lens| {
            lens.command == "versionlens.suggestion.onUpdateDependency"
                && lens.arguments.first() == Some(&suggestion.dependency.name)
                && lens.arguments.len() >= 4
        })
        .map(|lens| versionlens_suggestions::semantic_update_target(&lens.arguments[3]))
        .collect::<Vec<_>>();
    assert_eq!(
        targets.iter().collect::<BTreeSet<_>>().len(),
        targets.len(),
        "{case} emitted semantically duplicate update targets: {suggestion:?}"
    );
    if suggestion.status == "satisfiesLatest"
        && let Some(latest) = suggestion.latest.as_deref()
    {
        let latest = versionlens_suggestions::semantic_update_target(latest);
        assert!(
            !targets.contains(&latest),
            "{case} emitted a status and update action for the same target: {suggestion:?}"
        );
    }
}

fn assert_manifest_input(
    session: &VersionLensSession,
    input: &DocumentInput,
    case: &ManifestCase,
) -> versionlens_model::Dependency {
    assert_eq!(
        input.text, case.text,
        "{} discovery changed text",
        case.kind
    );
    let kind = session.classify_document(input);
    assert!(
        versionlens_model::provider_name_for_manifest(kind).is_some(),
        "{} classified unsupported",
        case.kind
    );
    assert_eq!(format!("{kind:?}"), case.kind);
    let dependencies = session.dependencies(input);
    let dependency = dependencies
        .iter()
        .find(|dependency| {
            (dependency.name.as_str(), dependency.requirement.as_str())
                == (
                    case.dependency.package.as_str(),
                    case.dependency.requirement.as_str(),
                )
        })
        .unwrap_or_else(|| {
            panic!(
                "{} did not parse {} {}: {dependencies:?}",
                case.kind, case.dependency.package, case.dependency.requirement,
            )
        });
    assert_eq!(
        format!("{:?}", dependency.ecosystem),
        case.dependency.ecosystem
    );
    if dependency.versionable_kind() == VersionableKind::ProjectVersion {
        assert!(
            session.document_is_fresh(input),
            "{} project result was not immediately fresh",
            case.kind
        );
    } else {
        assert!(
            !session.document_is_fresh(input),
            "{} was fresh before checking",
            case.kind
        );
    }
    let analysis = session.analyze_document(input.clone());
    assert!(analysis.is_supported_manifest, "{} unsupported", case.kind);
    assert_eq!(
        analysis.active_provider_name.as_deref(),
        versionlens_model::provider_name_for_manifest(kind)
    );
    dependency.clone()
}

struct ManifestEditCheck<'a> {
    session: &'a VersionLensSession,
    input: &'a DocumentInput,
    case: &'a ManifestCase,
    dependency: &'a versionlens_model::Dependency,
    response: &'a RegistryResponseInput,
    resolved: &'a crate::contract::ResolveDocumentOutput,
}

fn assert_manifest_edits(check: ManifestEditCheck<'_>) {
    let ManifestEditCheck {
        session,
        input,
        case,
        dependency,
        response,
        resolved,
    } = check;
    let selected_output = case.resolution.selected.as_deref().map(|selected| {
        session.apply_command_with_selected_version(super::ApplyCommandRequest {
            input: input.clone(),
            command: Some("update"),
            dependency_name: Some(case.dependency.package.as_str()),
            selected_version: Some(selected),
            responses: std::slice::from_ref(response),
        })
    });
    let edits = selected_output
        .as_ref()
        .map_or(&resolved.edits, |output| &output.edits);
    if case.resolution.selected.is_some() {
        assert!(
            resolved
                .edits
                .iter()
                .all(|edit| edit.range != dependency.requirement_range),
            "{} silently edited a fixed pin",
            case.kind
        );
    }
    match case.resolution.edit.as_deref() {
        Some(expected) => {
            let edit = edits
                .iter()
                .find(|edit| edit.range == dependency.requirement_range)
                .unwrap_or_else(|| panic!("{} omitted its update edit", case.kind));
            assert_eq!(edit.new_text, expected, "{}", case.kind);
            assert_eq!(
                slice_range(&input.text, edit),
                case.dependency
                    .range_text
                    .as_deref()
                    .unwrap_or(&case.dependency.requirement),
                "{}",
                case.kind
            );
            let expected_document = case.resolution.expected_text.clone().unwrap_or_else(|| {
                apply_edits(
                    &input.text,
                    &[TextEdit {
                        range: dependency.requirement_range,
                        new_text: expected.to_owned(),
                    }],
                )
            });
            assert_eq!(
                apply_edits(&input.text, edits),
                expected_document,
                "{} final document",
                case.kind
            );
        }
        None => assert!(edits.is_empty(), "{} emitted an unsafe edit", case.kind),
    }
}

fn assert_authoritative_case_coverage(cases: &[ManifestCase]) {
    let expected = parser_dispatch_kind_names();
    let actual = cases
        .iter()
        .map(|case| case.kind.clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        actual.len(),
        cases.len(),
        "manifest coverage kinds must be unique"
    );
    assert_eq!(
        actual, expected,
        "coverage must follow the exhaustive parser/model dispatch set"
    );
}

fn assert_editor_selector_coverage(cases: &[ManifestCase]) {
    let matrix: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../../tests/fixtures/manifest-support-matrix.json"
    ))
    .unwrap();
    let selected = matrix["entries"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|entry| entry["manifests"].as_array().unwrap())
        .flat_map(|manifest| manifest["manifestKinds"].as_array().unwrap())
        .filter_map(serde_json::Value::as_str)
        .collect::<BTreeSet<_>>();
    for case in cases {
        assert!(
            selected.contains(case.kind.as_str()),
            "{} has no editor selector",
            case.kind
        );
        let has_language = matrix["entries"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|entry| entry["manifests"].as_array().unwrap())
            .filter(|manifest| {
                manifest["manifestKinds"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|kind| kind.as_str() == Some(case.kind.as_str()))
            })
            .flat_map(|manifest| manifest["languages"].as_array().unwrap())
            .any(|language| language.as_str() == Some(case.language.as_str()));
        assert!(
            has_language,
            "{} has no {} editor selector",
            case.kind, case.language
        );
    }
}
