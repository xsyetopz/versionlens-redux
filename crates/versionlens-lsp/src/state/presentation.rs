use lsp_types::{
    CodeLens, Command, Diagnostic, DiagnosticSeverity, NumberOrString, Position, Range, Uri,
};
use versionlens_model::{Position as ModelPosition, Range as ModelRange};
use versionlens_vscode_model::{CodeLensPayload, DependencyPayload, DiagnosticPayload};

use super::{DISPLAY_CODE_LENS_COMMAND, UPDATE_DEPENDENCY_COMMAND};

pub fn into_lsp_range(range: ModelRange) -> Range {
    Range {
        start: Position::new(range.start.line, range.start.character),
        end: Position::new(range.end.line, range.end.character),
    }
}

pub(super) fn into_lsp_code_lenses(
    mut payload: CodeLensPayload,
    dependencies: &[DependencyPayload],
    uri: &str,
    generation: u64,
) -> Vec<CodeLens> {
    if let Some(dependency) = dependencies
        .iter()
        .find(|dependency| range_contains(dependency.range, payload.range.start))
    {
        payload.range = dependency_source_range(dependency);
    }
    if payload.command == "versionlens.suggestion.onChooseBuild" {
        return payload
            .arguments
            .iter()
            .skip(3)
            .map(|version| {
                command_lens(
                    payload.range,
                    format!("{} {version}", payload.title),
                    uri,
                    generation,
                    vec![
                        payload.arguments[1].clone(),
                        payload.arguments[0].clone(),
                        "update".to_owned(),
                        version.clone(),
                    ],
                )
            })
            .collect();
    }
    if payload.command == UPDATE_DEPENDENCY_COMMAND {
        return vec![command_lens(
            payload.range,
            payload.title,
            uri,
            generation,
            payload.arguments,
        )];
    }
    vec![CodeLens {
        range: into_lsp_range(payload.range),
        command: Some(Command {
            title: payload.title,
            command: DISPLAY_CODE_LENS_COMMAND.to_owned(),
            arguments: None,
        }),
        data: None,
    }]
}

fn range_contains(range: ModelRange, position: ModelPosition) -> bool {
    position_key(position) >= position_key(range.start)
        && position_key(position) <= position_key(range.end)
}

fn dependency_source_range(dependency: &DependencyPayload) -> ModelRange {
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

pub(super) fn into_lsp_diagnostic(payload: DiagnosticPayload) -> Diagnostic {
    Diagnostic {
        range: into_lsp_range(payload.range),
        severity: diagnostic_severity(payload.severity),
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

fn diagnostic_severity(severity: u8) -> Option<DiagnosticSeverity> {
    match severity {
        0 => Some(DiagnosticSeverity::ERROR),
        1 => Some(DiagnosticSeverity::WARNING),
        2 => Some(DiagnosticSeverity::INFORMATION),
        3 => Some(DiagnosticSeverity::HINT),
        _ => None,
    }
}
