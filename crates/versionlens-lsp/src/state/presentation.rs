use lsp_types::{
    CodeLens, Command, Diagnostic, DiagnosticSeverity, NumberOrString, Position, Range, Uri,
};
use versionlens_engine::{
    Action, BumpActionView, DependencyView, DiagnosticView, LensView, Severity, UpdateMode,
};
use versionlens_model::{Position as ModelPosition, Range as ModelRange};

use super::{CHOOSE_BUILD_COMMAND, DISPLAY_CODE_LENS_COMMAND, UPDATE_COMMAND};

pub fn into_lsp_range(range: ModelRange) -> Range {
    Range {
        start: Position::new(range.start.line, range.start.character),
        end: Position::new(range.end.line, range.end.character),
    }
}

pub(super) fn into_lsp_code_lenses(
    lens: LensView,
    dependencies: &[DependencyView],
    uri: &str,
) -> Vec<CodeLens> {
    let range = dependencies
        .iter()
        .find(|dependency| range_contains(dependency.range, lens.range.start))
        .map_or(lens.range, dependency_source_range);
    match lens.action {
        Some(Action::ChooseBuild {
            selector, builds, ..
        }) => builds
            .into_iter()
            .map(|version| {
                command_lens(
                    range,
                    format!("{} {version}", lens.title),
                    CHOOSE_BUILD_COMMAND,
                    serde_json::json!({"uri": uri, "dependency": selector, "build": version}),
                )
            })
            .collect(),
        Some(Action::UpdateDependency {
            selector,
            mode,
            version,
            ..
        }) => {
            let mut arguments = serde_json::json!({"uri": uri, "dependency": selector});
            if let Some(level) = mode.map(update_level) {
                arguments["level"] = level.into();
            }
            if let Some(version) = version {
                arguments["version"] = version.into();
            }
            vec![command_lens(range, lens.title, UPDATE_COMMAND, arguments)]
        }
        Some(Action::OpenFile { .. }) | None => vec![CodeLens {
            range: into_lsp_range(range),
            command: Some(Command {
                title: lens.title,
                command: DISPLAY_CODE_LENS_COMMAND.to_owned(),
                arguments: None,
            }),
            data: None,
        }],
    }
}

fn range_contains(range: ModelRange, position: ModelPosition) -> bool {
    position_key(position) >= position_key(range.start)
        && position_key(position) <= position_key(range.end)
}

fn dependency_source_range(dependency: &DependencyView) -> ModelRange {
    ModelRange {
        start: if position_key(dependency.requirement_range.start)
            < position_key(dependency.range.start)
        {
            dependency.requirement_range.start
        } else {
            dependency.range.start
        },
        end: if position_key(dependency.requirement_range.end) > position_key(dependency.range.end)
        {
            dependency.requirement_range.end
        } else {
            dependency.range.end
        },
    }
}

fn position_key(position: ModelPosition) -> (u32, u32) {
    (position.line, position.character)
}

/// The `level` argument of the update command for a lens mode.
const fn update_level(mode: UpdateMode) -> &'static str {
    match mode {
        UpdateMode::Update => "latest",
        UpdateMode::UpdateMajor => "major",
        UpdateMode::UpdateMinor => "minor",
        UpdateMode::UpdatePatch => "patch",
        UpdateMode::UpdateRelease => "release",
        UpdateMode::UpdatePrerelease => "prerelease",
    }
}

fn command_lens(
    range: ModelRange,
    title: String,
    command: &str,
    arguments: serde_json::Value,
) -> CodeLens {
    CodeLens {
        range: into_lsp_range(range),
        command: Some(Command {
            title,
            command: command.to_owned(),
            arguments: Some(vec![arguments]),
        }),
        data: None,
    }
}

pub(super) fn into_lsp_diagnostic(payload: DiagnosticView) -> Diagnostic {
    Diagnostic {
        range: into_lsp_range(payload.range),
        severity: Some(diagnostic_severity(payload.severity)),
        code: payload.code.map(NumberOrString::String),
        code_description: payload
            .code_description_url
            .and_then(|href| href.parse::<Uri>().ok())
            .map(|href| lsp_types::CodeDescription { href }),
        source: payload.source,
        message: payload.message,
        related_information: None,
        tags: None,
        data: None,
    }
}

const fn diagnostic_severity(severity: Severity) -> DiagnosticSeverity {
    match severity {
        Severity::Error => DiagnosticSeverity::ERROR,
        Severity::Warning => DiagnosticSeverity::WARNING,
        Severity::Information => DiagnosticSeverity::INFORMATION,
        Severity::Hint => DiagnosticSeverity::HINT,
    }
}

/// The quick fixes of a document, each carrying its edit inline. A client
/// that takes document changes gets a versioned edit, so it rejects the edit
/// for a newer document version.
pub(crate) fn into_lsp_code_actions(
    actions: Vec<BumpActionView>,
    uri: &Uri,
    version: Option<i32>,
    document_changes: bool,
) -> Vec<lsp_types::CodeAction> {
    actions
        .into_iter()
        .map(|action| {
            let edit = lsp_types::TextEdit {
                range: into_lsp_range(action.edit.range),
                new_text: action.edit.new_text,
            };
            let edit = if document_changes {
                lsp_types::WorkspaceEdit {
                    document_changes: Some(lsp_types::DocumentChanges::Edits(vec![
                        lsp_types::TextDocumentEdit {
                            text_document: lsp_types::OptionalVersionedTextDocumentIdentifier {
                                uri: uri.clone(),
                                version,
                            },
                            edits: vec![lsp_types::OneOf::Left(edit)],
                        },
                    ])),
                    ..lsp_types::WorkspaceEdit::default()
                }
            } else {
                lsp_types::WorkspaceEdit {
                    changes: Some(std::iter::once((uri.clone(), vec![edit])).collect()),
                    ..lsp_types::WorkspaceEdit::default()
                }
            };
            lsp_types::CodeAction {
                title: action.title,
                kind: Some(lsp_types::CodeActionKind::QUICKFIX),
                diagnostics: action
                    .diagnostic
                    .map(|diagnostic| vec![into_lsp_diagnostic(diagnostic)]),
                edit: Some(edit),
                is_preferred: Some(action.preferred),
                ..lsp_types::CodeAction::default()
            }
        })
        .collect()
}
