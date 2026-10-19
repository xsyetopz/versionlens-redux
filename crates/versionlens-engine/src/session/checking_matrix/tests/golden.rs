use std::fs;
use std::path::PathBuf;

use serde_json::{Value, json};
use versionlens_model::DocumentInput;

use super::super::ApplyCommandRequest;
use super::{ManifestCase, manifest_cases, response_body, test_session};
use crate::workspace::tests::support::TestWorkspace;
use crate::{Action, DiagnosticView, LensView, RegistryResponseInput, Severity, UpdateMode};

/// Writes the lens, diagnostic and edit output of every manifest case to
/// `tests/golden/<release>/`, where the release is the case's `since`, or
/// 0.4.4 for kinds that release supported. Run with
/// `cargo test -p versionlens-engine capture_golden_output -- --ignored`.
#[tokio::test]
#[ignore = "writes tests/golden"]
async fn capture_golden_output() {
    let golden = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/golden");
    for case in manifest_cases() {
        let release = case.since.as_deref().unwrap_or("0.4.4");
        let output = golden.join(release).join(format!("{}.json", case.path));
        fs::create_dir_all(output.parent().unwrap()).unwrap();
        fs::write(output, capture_case(&case).await).unwrap();
    }
}

/// One manifest case checked against its canned registry response, with the
/// command of every lens run against the same response.
struct CaseRun {
    root_uri: String,
    root: String,
    text: String,
    analysis: crate::AnalyzeDocumentOutput,
    resolved: crate::ResolveDocumentOutput,
    actions: Vec<LensRun>,
}

struct LensRun {
    lens: String,
    command: String,
    dependency: String,
    selected_version: Option<String>,
    output: crate::ResolveDocumentOutput,
}

async fn run_case(case: &ManifestCase) -> CaseRun {
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
            actions.push(LensRun {
                lens: lens.title.clone(),
                command: command.to_owned(),
                dependency: dependency_name.to_owned(),
                selected_version: selected_version.map(ToOwned::to_owned),
                output,
            });
        }
    }
    let root = workspace.path().to_string_lossy().into_owned();
    fs::remove_dir_all(workspace.path()).unwrap();
    CaseRun {
        root_uri,
        root,
        text: case.text.clone(),
        analysis,
        resolved,
        actions,
    }
}

async fn capture_case(case: &ManifestCase) -> String {
    let run = run_case(case).await;
    let actions = run
        .actions
        .iter()
        .map(|action| {
            json!({
                "lens": action.lens,
                "command": action.command,
                "dependency": action.dependency,
                "selectedVersion": action.selected_version,
                "edits": action.output.edits,
            })
        })
        .collect::<Vec<_>>();
    let golden = json!({
        "kind": case.kind,
        "path": case.path,
        "dependencies": run.analysis.dependencies,
        "codeLenses": run.analysis.code_lenses.iter().map(golden_code_lens).collect::<Vec<_>>(),
        // Update diagnostics have their own tests. Leaving them out means a
        // golden file changes only when lens, edit or vulnerability output does.
        "diagnostics": run
            .analysis
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.code.as_deref() != Some(crate::UPDATE_AVAILABLE))
            .map(golden_diagnostic)
            .collect::<Vec<_>>(),
        "updateAllEdits": run.resolved.edits,
        "actions": actions,
    });
    let text = serde_json::to_string_pretty(&golden).unwrap() + "\n";
    let text = text.replace(&run.root_uri, "file:///workspace");
    text.replace(&run.root, "/workspace")
}

/// Every lens that offers an update must change the document when clicked.
/// A lens whose command has no edit, or only edits that keep the text, is a
/// no-op and must not be shown.
#[tokio::test]
async fn every_update_lens_changes_the_document() {
    let mut noops = Vec::new();
    for case in manifest_cases() {
        let run = run_case(&case).await;
        for action in &run.actions {
            let planned = action.output.edit_plan.as_ref().is_some_and(|plan| {
                plan.documents
                    .iter()
                    .any(|document| !document.edits.is_empty())
            });
            let changes = action
                .output
                .edits
                .iter()
                .any(|edit| text_at(&run.text, edit.range) != edit.new_text);
            if !planned && !changes {
                noops.push(format!(
                    "{}: {:?} ({})",
                    case.path, action.lens, action.command
                ));
            }
        }
    }
    assert!(noops.is_empty(), "no-op lenses:\n{}", noops.join("\n"));
}

/// The text of `range`, counting characters as UTF-16 units like LSP does.
fn text_at(text: &str, range: versionlens_model::Range) -> String {
    let offset = |position: versionlens_model::Position| {
        let line_start: usize = text
            .split_inclusive('\n')
            .take(position.line as usize)
            .map(str::len)
            .sum();
        let mut units = 0;
        let line = &text[line_start..];
        let within = line
            .char_indices()
            .find(|(_, character)| {
                let reached = units >= position.character as usize;
                units += character.len_utf16();
                reached
            })
            .map_or(line.len(), |(index, _)| index);
        line_start + within
    };
    text[offset(range.start)..offset(range.end)].to_owned()
}

/// Converts a lens to the golden `CodeLensPayload` shape: a command ID,
/// positional arguments, and a point range moved `order` characters right,
/// because VS Code sorts the lenses of a line by position.
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

/// The VS Code command ID of the golden format for this lens action.
fn golden_command(lens: &LensView) -> &'static str {
    match lens.action_kind() {
        "UpdateDependency" => "versionlens.suggestion.onUpdateDependency",
        "ChooseBuild" => "versionlens.suggestion.onChooseBuild",
        "OpenFile" => "versionlens.suggestion.onFileLink",
        _ => "",
    }
}

/// Converts a diagnostic to the golden `DiagnosticPayload` shape, whose severity is a number.
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

/// Commit that first captured `tests/golden/0.4.4` from the napi build.
const BASELINE_COMMIT: &str = "67f3b1d4";

fn golden_path(case: &ManifestCase) -> PathBuf {
    let release = case.since.as_deref().unwrap_or("0.4.4");
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/golden")
        .join(release)
        .join(format!("{}.json", case.path))
}

/// The engine output of every manifest case equals its file in `tests/golden`.
#[tokio::test]
async fn engine_output_matches_the_golden_folder() {
    let mut stale = Vec::new();
    for case in manifest_cases() {
        let expected = fs::read_to_string(golden_path(&case)).unwrap_or_default();
        if capture_case(&case).await != expected {
            stale.push(case.path.clone());
        }
    }
    assert!(
        stale.is_empty(),
        "golden files out of date:\n{}",
        stale.join("\n")
    );
}

/// `tests/golden/0.4.4` differs from the 67f3b1d4 capture exactly in the files
/// that `tests/golden/DIFFERENCES.md` lists under a `### <path>` heading: no
/// unlisted difference, no stale entry, no file missing from the baseline. The
/// check panics under `CI` and is skipped locally where git or that commit is
/// unavailable.
#[test]
fn golden_differences_with_the_baseline_are_documented() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/golden");
    let documented = fs::read_to_string(root.join("DIFFERENCES.md")).unwrap();
    let listed: Vec<&str> = documented
        .lines()
        .filter_map(|line| line.strip_prefix("### `")?.strip_suffix('`'))
        .collect();
    let git = |args: &[&str]| {
        std::process::Command::new("git")
            .current_dir(&root)
            .args(args)
            .output()
            .ok()
            .filter(|output| output.status.success())
    };
    let Some(files) = git(&["ls-tree", "-r", "--name-only", BASELINE_COMMIT, "0.4.4"]) else {
        assert!(
            std::env::var_os("CI").is_none(),
            "git history for {BASELINE_COMMIT} is unavailable; fetch with depth 0"
        );
        eprintln!("skipping baseline comparison: {BASELINE_COMMIT} is unavailable");
        return;
    };
    let prefix = git(&["rev-parse", "--show-prefix"]).map(|o| String::from_utf8(o.stdout).unwrap());
    let prefix = prefix.unwrap_or_default().trim().to_owned();
    let baseline_files: Vec<String> = String::from_utf8(files.stdout)
        .unwrap()
        .lines()
        .map(|file| file.trim_start_matches("0.4.4/").to_owned())
        .collect();
    let mut problems = Vec::new();
    for name in &baseline_files {
        let baseline = git(&["show", &format!("{BASELINE_COMMIT}:{prefix}0.4.4/{name}")]);
        let current = fs::read(root.join("0.4.4").join(name)).ok();
        let same = baseline.map(|o| o.stdout) == current;
        let is_listed = listed.contains(&name.as_str());
        if !same && !is_listed {
            problems.push(format!("undocumented difference: {name}"));
        }
        if (same || current.is_none()) && is_listed {
            problems.push(format!("stale entry (identical or missing): {name}"));
        }
    }
    for entry in &listed {
        if !baseline_files.iter().any(|name| name == entry) {
            problems.push(format!("listed but not in the baseline: {entry}"));
        }
    }
    let mut pending = vec![root.join("0.4.4")];
    while let Some(dir) = pending.pop() {
        for entry in fs::read_dir(dir)
            .unwrap()
            .map(|entry| entry.unwrap().path())
        {
            if entry.is_dir() {
                pending.push(entry);
                continue;
            }
            let name = entry
                .strip_prefix(root.join("0.4.4"))
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            if !baseline_files.contains(&name) {
                problems.push(format!("not in the baseline: {name}"));
            }
        }
    }
    assert!(
        problems.is_empty(),
        "0.4.4 golden files disagree with {BASELINE_COMMIT} and DIFFERENCES.md:\n{}",
        problems.join("\n")
    );
}
