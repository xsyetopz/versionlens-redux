use versionlens_parsers::Dependency;
use versionlens_suggestions::{Suggestion, SuggestionStatus, UpdateChoice};
use versionlens_vscode_model::CodeLensPayload;

use crate::SuggestionIndicators;
use crate::project::ProjectVersionCodeLensSuggestion;
use crate::selection::dependency_selector;

mod diagnostics;
mod indicator;
mod title;

pub(crate) use diagnostics::vulnerability_diagnostics;

pub(crate) fn code_lens_payload(dependency: &Dependency, title: String) -> CodeLensPayload {
    CodeLensPayload {
        range: dependency.range,
        title,
        command: "versionlens.suggestion.onUpdateDependency".to_owned(),
        arguments: vec![
            dependency.name.as_str().to_owned(),
            dependency_selector(dependency),
        ],
    }
}

pub(crate) fn project_version_code_lens_payload(
    dependency: &Dependency,
    suggestion: ProjectVersionCodeLensSuggestion,
    indicators: &SuggestionIndicators,
) -> CodeLensPayload {
    let title = indicator::with_indicator(
        &indicators.updateable,
        format!("{} {}", suggestion.label, suggestion.latest),
    );
    CodeLensPayload {
        range: dependency.range,
        title,
        command: "versionlens.suggestion.onUpdateDependency".to_owned(),
        arguments: vec![
            dependency.name.as_str().to_owned(),
            dependency_selector(dependency),
            suggestion.command.to_owned(),
        ],
    }
}

pub(crate) fn suggested_code_lens_payload(
    dependency: &Dependency,
    suggestion: Option<&Suggestion>,
    title: String,
) -> CodeLensPayload {
    if let Some(suggestion) = suggestion
        && suggestion.status == SuggestionStatus::BuildAvailable
        && let Some(builds) = build_choices(suggestion)
    {
        let mut arguments = vec![
            dependency_selector(dependency),
            dependency.name.as_str().to_owned(),
            dependency.requirement.as_str().to_owned(),
        ];
        arguments.extend(builds);
        return CodeLensPayload {
            range: dependency.range,
            title,
            command: "versionlens.suggestion.onChooseBuild".to_owned(),
            arguments,
        };
    }

    if let Some(path) = directory_open_path(suggestion) {
        return CodeLensPayload {
            range: dependency.range,
            title,
            command: "versionlens.suggestion.onFileLink".to_owned(),
            arguments: vec![path.to_owned()],
        };
    }

    code_lens_payload(dependency, title)
}

pub(crate) fn update_choice_code_lens_payload(
    dependency: &Dependency,
    choice: &UpdateChoice,
    indicators: &SuggestionIndicators,
    has_vulnerabilities: bool,
) -> CodeLensPayload {
    let title = indicator::with_indicator(
        indicator::update_indicator(indicators, has_vulnerabilities),
        format!("{} {}", choice.label.as_str(), choice.version.as_str()),
    );
    CodeLensPayload {
        range: dependency.range,
        title,
        command: "versionlens.suggestion.onUpdateDependency".to_owned(),
        arguments: vec![
            dependency.name.as_str().to_owned(),
            dependency_selector(dependency),
            choice.command.as_str().to_owned(),
            choice.version.as_str().to_owned(),
        ],
    }
}

pub(crate) fn code_lens_title(
    dependency: &Dependency,
    suggestion: Option<&Suggestion>,
    indicators: &SuggestionIndicators,
    has_vulnerabilities: bool,
) -> String {
    let title = title::code_lens_title_text(dependency, suggestion);
    let indicator = indicator::code_lens_indicator(indicators, suggestion, has_vulnerabilities);
    indicator::with_indicator(indicator, title)
}

fn directory_open_path(suggestion: Option<&Suggestion>) -> Option<&str> {
    let suggestion = suggestion?;
    (suggestion.status == SuggestionStatus::Directory)
        .then_some(suggestion.resolved.as_deref())
        .flatten()
}

fn build_choices(suggestion: &Suggestion) -> Option<Vec<String>> {
    if !suggestion.builds.is_empty() {
        return Some(suggestion.builds.iter().map(String::to_owned).collect());
    }

    suggestion
        .latest
        .as_ref()
        .map(|latest| vec![latest.to_owned()])
}
