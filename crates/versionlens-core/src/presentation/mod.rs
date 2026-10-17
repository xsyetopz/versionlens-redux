use self::indicator::{code_lens_indicator, update_indicator, with_indicator};
use self::title::code_lens_title_text;
use versionlens_model::Dependency;
use versionlens_model::Ecosystem::GitHub;
use versionlens_suggestions::SuggestionStatus::{
    BuildAvailable as StatusBuildAvailable, Directory as StatusDirectory,
};
use versionlens_suggestions::{Suggestion, UpdateChoice};

use crate::SuggestionIndicators;
use crate::project::ProjectVersionCodeLensSuggestion;
use crate::selection::dependency_selector;
use crate::view::{Action, LensView, UpdateMode};

mod diagnostics;
mod indicator;
pub(crate) mod statuses;
mod title;

pub(crate) use diagnostics::vulnerability_diagnostics;

type SuggestionRef<'a> = Option<&'a Suggestion>;

pub(crate) fn code_lens_view(dependency: &Dependency, title: String) -> LensView {
    LensView::new(
        dependency.range,
        title,
        Some(Action::UpdateDependency {
            name: dependency.name.as_str().to_owned(),
            selector: dependency_selector(dependency),
            mode: None,
            version: None,
        }),
    )
}

pub(crate) fn project_version_code_lens_view(
    dependency: &Dependency,
    suggestion: ProjectVersionCodeLensSuggestion,
    indicators: &SuggestionIndicators,
) -> LensView {
    let title = with_indicator(
        &indicators.updateable,
        format!("{} {}", suggestion.label, suggestion.latest),
    );
    LensView::new(
        dependency.range,
        title,
        Some(Action::UpdateDependency {
            name: dependency.name.as_str().to_owned(),
            selector: dependency_selector(dependency),
            mode: Some(suggestion.command),
            version: None,
        }),
    )
}

pub(crate) fn suggested_code_lens_view(
    dependency: &Dependency,
    suggestion: SuggestionRef<'_>,
    title: String,
) -> LensView {
    if let Some(suggestion) = suggestion
        && suggestion.status == StatusBuildAvailable
        && let Some(builds) = build_choices(suggestion)
    {
        return LensView::new(
            dependency.range,
            title,
            Some(Action::ChooseBuild {
                selector: dependency_selector(dependency),
                name: dependency.name.as_str().to_owned(),
                requirement: dependency.requirement.as_str().to_owned(),
                builds,
            }),
        );
    }

    if let Some(path) = directory_open_path(suggestion) {
        return LensView::new(
            dependency.range,
            title,
            Some(Action::OpenFile {
                path: path.to_owned(),
            }),
        );
    }

    code_lens_view(dependency, title)
}

pub(crate) fn update_choice_code_lens_view(
    dependency: &Dependency,
    choice: &UpdateChoice,
    indicators: &SuggestionIndicators,
    has_vulnerabilities: bool,
) -> LensView {
    let title = with_indicator(
        choice_indicator(choice, indicators, has_vulnerabilities),
        format!(
            "{} {}",
            choice.label.as_str(),
            displayed_choice_version(dependency, choice)
        ),
    );
    // A stored choice whose command is not a known mode yields a display-only lens.
    let action = UpdateMode::parse(choice.command.as_str()).map(|mode| Action::UpdateDependency {
        name: dependency.name.as_str().to_owned(),
        selector: dependency_selector(dependency),
        mode: Some(mode),
        version: Some(choice.version.as_str().to_owned()),
    });
    LensView::new(dependency.range, title, action)
}

fn displayed_choice_version(dependency: &Dependency, choice: &UpdateChoice) -> String {
    if dependency.ecosystem == GitHub
        && !dependency.requirement_prefix.is_empty()
        && !choice.version.starts_with(&dependency.requirement_prefix)
    {
        return format!("{}{}", dependency.requirement_prefix, choice.version);
    }
    choice.version.clone()
}

fn choice_indicator<'a>(
    _choice: &UpdateChoice,
    indicators: &'a SuggestionIndicators,
    has_vulnerabilities: bool,
) -> &'a str {
    if has_vulnerabilities {
        return update_indicator(indicators, true);
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
