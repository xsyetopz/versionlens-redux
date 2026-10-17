use std::fs;
use std::path::PathBuf;

use serde_json::{Value, json};
use versionlens_model::DocumentInput;

use super::super::ApplyCommandRequest;
use super::{ManifestCase, manifest_cases, response_body, test_session};
use crate::workspace::tests::support::TestWorkspace;
use crate::{Action, DiagnosticView, LensView, RegistryResponseInput, Severity, UpdateMode};

/// Writes the lens, diagnostic and edit output of every manifest case to
/// `tests/golden/0.4.4/`. Run with
/// `cargo test -p versionlens-core capture_golden_output -- --ignored`.
#[tokio::test]
#[ignore = "writes tests/golden/0.4.4"]
async fn capture_golden_output() {
    let golden = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/golden/0.4.4");
    for case in manifest_cases() {
        let output = golden.join(format!("{}.json", case.path));
        fs::create_dir_all(output.parent().unwrap()).unwrap();
        fs::write(output, capture_case(&case).await).unwrap();
    }
}

async fn capture_case(case: &ManifestCase) -> String {
    let workspace = TestWorkspace::new("golden");
    let root_uri = crate::workspace_file_uri(workspace.path()).unwrap();
    let input = DocumentInput::new(
        crate::workspace_file_uri(&workspace.write(&case.path, &case.text)).unwrap(),
        &case.language,
        &case.text,
        Some(root_uri.clone()),
    );
    let session = test_session();
    let ecosystem = session
        .dependencies(&input)
        .into_iter()
        .find(|dependency| dependency.name == case.dependency.package)
        .unwrap_or_else(|| panic!("{} did not parse {}", case.kind, case.dependency.package))
        .ecosystem;
    let responses = [RegistryResponseInput::new(
        case.dependency
            .response_package
            .as_deref()
            .unwrap_or(&case.dependency.package),
        ecosystem,
        response_body(case),
    )];
    let resolved = session
        .resolve_document_with_responses(input.clone(), &responses)
        .await;
    let analysis = session.analyze_document(input.clone());
    let mut actions = Vec::new();
    for lens in &analysis.code_lenses {
        for (command, dependency_name, selected_version) in lens_actions(lens) {
            let output = session
                .apply_command_with_selected_version(ApplyCommandRequest {
                    input: input.clone(),
                    command: Some(command),
                    dependency_name: Some(dependency_name),
                    selected_version,
                    responses: &responses,
                })
                .await;
            actions.push(json!({
                "lens": lens.title,
                "command": command,
                "dependency": dependency_name,
                "selectedVersion": selected_version,
                "edits": output.edits,
            }));
        }
    }
    let golden = json!({
        "kind": case.kind,
        "path": case.path,
        "dependencies": analysis.dependencies,
        "codeLenses": analysis.code_lenses.iter().map(golden_code_lens).collect::<Vec<_>>(),
        "diagnostics": analysis.diagnostics.iter().map(golden_diagnostic).collect::<Vec<_>>(),
        "updateAllEdits": resolved.edits,
        "actions": actions,
    });
    let text = serde_json::to_string_pretty(&golden).unwrap() + "\n";
    let text = text.replace(&root_uri, "file:///workspace");
    let root = workspace.path().to_string_lossy();
    let text = text.replace(root.as_ref(), "/workspace");
    fs::remove_dir_all(workspace.path()).unwrap();
    text
}

/// Converts a lens to the 0.4.4 `CodeLensPayload` shape: a command ID and
/// positional arguments. 0.4.4 kept lens order by moving each lens one
/// character right of the previous one, collapsed to a point.
fn golden_code_lens(lens: &LensView) -> Value {
    let mut range = lens.range;
    range.start.character = range
        .start
        .character
        .saturating_add(lens.order)
        .min(lens.range.end.character);
    range.end = range.start;
    json!({
        "range": range,
        "title": lens.title,
        "command": golden_command(lens),
        "arguments": lens.arguments(),
    })
}

/// The VS Code command ID that 0.4.4 put on a lens with this action.
fn golden_command(lens: &LensView) -> &'static str {
    match lens.action_kind() {
        "UpdateDependency" => "versionlens.suggestion.onUpdateDependency",
        "ChooseBuild" => "versionlens.suggestion.onChooseBuild",
        "OpenFile" => "versionlens.suggestion.onFileLink",
        _ => "",
    }
}

/// Converts a diagnostic to the 0.4.4 `DiagnosticPayload` shape, whose severity is a number.
fn golden_diagnostic(diagnostic: &DiagnosticView) -> Value {
    let severity = match diagnostic.severity {
        Severity::Error => 0,
        Severity::Warning => 1,
        Severity::Information => 2,
        Severity::Hint => 3,
    };
    json!({
        "range": diagnostic.range,
        "message": diagnostic.message,
        "severity": severity,
        "source": diagnostic.source,
        "code": diagnostic.code,
        "codeDescriptionUrl": diagnostic.code_description_url,
    })
}

pub(super) fn lens_actions(lens: &LensView) -> Vec<(&str, &str, Option<&str>)> {
    match &lens.action {
        Some(Action::UpdateDependency {
            name,
            mode,
            version,
            ..
        }) => vec![(
            mode.map_or("update", UpdateMode::as_str),
            name.as_str(),
            version.as_deref(),
        )],
        Some(Action::ChooseBuild { name, builds, .. }) => builds
            .iter()
            .map(|build| ("update", name.as_str(), Some(build.as_str())))
            .collect(),
        Some(Action::OpenFile { .. }) | None => Vec::new(),
    }
}
