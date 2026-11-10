use self::indicator::{code_lens_indicator, update_indicator, with_indicator};
use self::title::{code_lens_title_text, prefixed_version};
use versionlens_model::Dependency;
use versionlens_update::{Suggestion, UpdateChoice};

use crate::SuggestionIndicators;
use crate::presentation::statuses::{StatusBuildAvailable, StatusDirectory, StatusUpdateAvailable};
use crate::presentation::view::{Action, LensView, UpdateMode};
use crate::project::ProjectVersionCodeLensSuggestion;
use crate::selection::dependency_selector;

mod indicator;
mod session;
pub(crate) mod status;
pub(crate) mod statuses;
pub(crate) mod suggestion;
mod title;
pub(crate) mod view;
mod vulnerability;

pub(crate) use vulnerability::vulnerability_diagnostics;

type SuggestionRef<'a> = Option<&'a Suggestion>;

fn update_action(
    dependency: &Dependency,
    mode: Option<UpdateMode>,
    version: Option<String>,
) -> Action {
    Action::UpdateDependency {
        name: dependency.name.as_str().to_owned(),
        selector: dependency_selector(dependency),
        mode,
        version,
    }
}

pub(crate) fn code_lens_view(dependency: &Dependency, title: String) -> LensView {
    LensView::new(
        dependency.range,
        title,
        Some(update_action(dependency, None, None)),
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
        Some(update_action(dependency, Some(suggestion.command), None)),
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

    // Only an available update has something to apply. Any other status
    // shows its title alone, because its command would change nothing.
    if suggestion.is_some_and(|suggestion| suggestion.status == StatusUpdateAvailable) {
        code_lens_view(dependency, title)
    } else {
        LensView::new(dependency.range, title, None)
    }
}

pub(crate) fn update_choice_code_lens_view(
    dependency: &Dependency,
    choice: &UpdateChoice,
    indicators: &SuggestionIndicators,
    has_vulnerabilities: bool,
) -> LensView {
    let title = with_indicator(
        update_indicator(indicators, has_vulnerabilities),
        format!(
            "{} {}",
            choice.label.as_str(),
            prefixed_version(dependency, &choice.version)
        ),
    );
    // A stored choice whose command is not a known mode yields a display-only lens.
    let action = UpdateMode::parse(choice.command.as_str()).map(|mode| {
        update_action(
            dependency,
            Some(mode),
            Some(choice.version.as_str().to_owned()),
        )
    });
    LensView::new(dependency.range, title, action)
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
        return Some(suggestion.builds.clone());
    }

    suggestion
        .latest
        .as_ref()
        .map(|latest| vec![latest.to_owned()])
}
