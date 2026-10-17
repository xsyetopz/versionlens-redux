use lsp_types::{
    CodeLens, Command, Diagnostic, DiagnosticSeverity, NumberOrString, Position, Range, Uri,
};
use versionlens_core::{
    Action, BumpActionView, DependencyView, DiagnosticView, LensView, Severity, UpdateMode,
};
use versionlens_model::{Position as ModelPosition, Range as ModelRange};

use super::{DISPLAY_CODE_LENS_COMMAND, UPDATE_DEPENDENCY_COMMAND};

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
    generation: u64,
) -> Vec<CodeLens> {
    let range = dependencies
        .iter()
        .find(|dependency| range_contains(dependency.range, lens.range.start))
        .map_or(lens.range, dependency_source_range);
    match lens.action {
        Some(Action::ChooseBuild {
            selector,
            name,
            builds,
            ..
        }) => builds
            .into_iter()
            .map(|version| {
                command_lens(
                    range,
                    format!("{} {version}", lens.title),
                    uri,
                    generation,
                    vec![
                        name.clone(),
                        selector.clone(),
                        UpdateMode::Update.as_str().to_owned(),
                        version,
                    ],
                )
            })
            .collect(),
        Some(Action::UpdateDependency {
            name,
            selector,
            mode,
            version,
        }) => {
            let mut arguments = vec![name, selector];
            arguments.extend(mode.map(|mode| mode.as_str().to_owned()));
            arguments.extend(version);
            vec![command_lens(range, lens.title, uri, generation, arguments)]
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

fn command_lens(
    range: ModelRange,
    title: String,
    uri: &str,
    generation: u64,
    arguments: Vec<String>,
) -> CodeLens {
    CodeLens {
        range: into_lsp_range(range),
        command: Some(Command {
            title,
            command: UPDATE_DEPENDENCY_COMMAND.to_owned(),
            arguments: Some(vec![
                serde_json::json!({"uri":uri,"generation":generation,"arguments":arguments}),
            ]),
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
