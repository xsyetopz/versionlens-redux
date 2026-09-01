use self::indicator::{code_lens_indicator, update_indicator, with_indicator};
use self::title::code_lens_title_text;
use versionlens_model::Dependency;
use versionlens_model::Ecosystem::GitHub;
use versionlens_suggestions::SuggestionStatus::{
    BuildAvailable as StatusBuildAvailable, Directory as StatusDirectory,
};
use versionlens_suggestions::{Suggestion, UpdateChoice};
use versionlens_vscode_model::CodeLensPayload;

use crate::SuggestionIndicators;
use crate::project::ProjectVersionCodeLensSuggestion;
use crate::selection::dependency_selector;

mod diagnostics;
mod indicator;
pub(crate) mod statuses;
mod title;

pub(crate) use diagnostics::vulnerability_diagnostics;

type SuggestionRef<'a> = Option<&'a Suggestion>;

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
    let title = with_indicator(
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
    suggestion: SuggestionRef<'_>,
    title: String,
) -> CodeLensPayload {
    if let Some(suggestion) = suggestion
        && suggestion.status == StatusBuildAvailable
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
    let title = with_indicator(
        choice_indicator(choice, indicators, has_vulnerabilities),
        format!(
            "{} {}",
            choice.label.as_str(),
            displayed_choice_version(dependency, choice)
        ),
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

fn displayed_choice_version(dependency: &Dependency, choice: &UpdateChoice) -> String {
    if dependency.ecosystem == GitHub
        && dependency.requirement_prefix == "v"
        && !choice.version.starts_with(['v', 'V'])
    {
        return format!("v{}", choice.version);
    }
    choice.version.clone()
}

fn choice_indicator<'a>(
    choice: &UpdateChoice,
    indicators: &'a SuggestionIndicators,
    has_vulnerabilities: bool,
) -> &'a str {
    if has_vulnerabilities {
        return update_indicator(indicators, true);
    }
    if choice.label == "downgrade" {
        return &indicators.downgradeable;
    }
    update_indicator(indicators, false)
}

pub(crate) fn code_lens_title(
    dependency: &Dependency,
    suggestion: SuggestionRef<'_>,
    indicators: &SuggestionIndicators,
    has_vulnerabilities: bool,
) -> String {
    let title = code_lens_title_text(dependency, suggestion);
    let indicator = code_lens_indicator(indicators, suggestion, has_vulnerabilities);
    with_indicator(indicator, title)
}

fn directory_open_path(suggestion: SuggestionRef<'_>) -> Option<&str> {
    let suggestion = suggestion?;
    (suggestion.status == StatusDirectory)
        .then_some(suggestion.resolved.as_deref())
        .flatten()
}

fn build_choices(suggestion: &Suggestion) -> Option<Vec<String>> {
    if !suggestion.builds.is_empty() {
        return Some(
            suggestion
                .builds
                .iter()
                .map(|value| value.to_owned())
                .collect(),
        );
    }

    suggestion
        .latest
        .as_ref()
        .map(|latest| vec![latest.to_owned()])
}
