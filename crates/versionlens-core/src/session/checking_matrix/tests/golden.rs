use std::fs;
use std::path::PathBuf;

use serde_json::json;
use versionlens_model::DocumentInput;

use super::super::ApplyCommandRequest;
use super::{ManifestCase, manifest_cases, response_body, test_session};
use crate::RegistryResponseInput;
use crate::workspace::tests::support::TestWorkspace;

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
        "codeLenses": analysis.code_lenses,
        "diagnostics": analysis.diagnostics,
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

pub(super) fn lens_actions(
    lens: &versionlens_vscode_model::CodeLensPayload,
) -> Vec<(&str, &str, Option<&str>)> {
    let argument = |index: usize| lens.arguments.get(index).map(String::as_str);
    match lens.command.as_str() {
        "versionlens.suggestion.onUpdateDependency" => vec![(
            argument(2).unwrap_or("update"),
            argument(0).unwrap(),
            argument(3),
        )],
        "versionlens.suggestion.onChooseBuild" => lens.arguments[3..]
            .iter()
            .map(|build| ("update", argument(1).unwrap(), Some(build.as_str())))
            .collect(),
        _ => Vec::new(),
    }
}
