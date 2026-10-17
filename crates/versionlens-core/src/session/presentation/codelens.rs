use crate::session::storage::CacheScope;
use versionlens_model::{Dependency, update_level};
use versionlens_update::Suggestion;

use crate::presentation::statuses::{
    StatusBuildAvailable, StatusCurrent, StatusDirectory, StatusDirectoryNotFound, StatusError,
    StatusFixed, StatusInvalid, StatusInvalidRange, StatusNoMatch, StatusNotSupported,
    StatusSatisfies, StatusSatisfiesLatest, StatusUpdateAvailable, SuggestionStatus,
};
use crate::presentation::{
    code_lens_title, project_version_code_lens_view, suggested_code_lens_view,
    update_choice_code_lens_view,
};
use crate::project::project_version_code_lens_suggestions;
use crate::view::LensView;
use crate::{SuggestionIndicators, VersionLensSession};
use versionlens_model::Ecosystem::Docker;

type CodeLensViews = Vec<LensView>;

impl VersionLensSession {
    pub(crate) fn code_lenses_for_dependency(
        &self,
        dependency: &Dependency,
        scope: &CacheScope,
    ) -> CodeLensViews {
        let project_version_suggestions = project_version_code_lens_suggestions(dependency);
        if !project_version_suggestions.is_empty() {
            return ordered_code_lenses(
                project_version_suggestions
                    .into_iter()
                    .map(|suggestion| {
                        project_version_code_lens_view(
                            dependency,
                            suggestion,
                            &self.config.suggestion_indicators,
                        )
                    })
                    .collect(),
            );
        }

        let Some(suggestion) = self.cached_suggestion(dependency, scope) else {
            return vec![];
        };

        if dependency.is_runtime_constraint() {
            return vec![runtime_constraint_lens(
                dependency,
                &suggestion,
                &self.config.suggestion_indicators,
            )];
        }

        let has_vulnerabilities = self.target_update_has_cached_vulnerabilities(Some(&suggestion));
        let title = code_lens_title(
            dependency,
            Some(&suggestion),
            &self.config.suggestion_indicators,
            has_vulnerabilities,
        );

        let mut lenses = status_only_code_lens(
            dependency,
            Some(&suggestion),
            &self.config.suggestion_indicators,
        )
        .into_iter()
        .collect::<Vec<_>>();

        if suggestion.status == StatusCurrent
            && dependency.ecosystem != Docker
            && suggestion.choices.is_empty()
        {
            if let Some(build_lens) = build_choice_code_lens_view(
                dependency,
                &suggestion,
                &self.config.suggestion_indicators,
                build_has_cached_vulnerabilities(self, dependency, &suggestion),
            ) {
                lenses.push(build_lens);
            }
            return ordered_code_lenses(lenses);
        }

        if shows_update_choice_lenses(suggestion.status) && !suggestion.choices.is_empty() {
            lenses.extend(suggestion.choices.iter().map(|choice| {
                update_choice_code_lens_view(
                    dependency,
                    choice,
                    &self.config.suggestion_indicators,
                    choice_has_cached_vulnerabilities(self, dependency, choice.version.as_str()),
                )
            }));
            if let Some(build_lens) = build_choice_code_lens_view(
                dependency,
                &suggestion,
                &self.config.suggestion_indicators,
                build_has_cached_vulnerabilities(self, dependency, &suggestion),
            ) {
                lenses.push(build_lens);
            }
            return ordered_code_lenses(lenses);
        }

        if let Some(build_lens) = build_choice_code_lens_view(
            dependency,
            &suggestion,
            &self.config.suggestion_indicators,
            build_has_cached_vulnerabilities(self, dependency, &suggestion),
        ) {
            lenses.push(build_lens);
            return ordered_code_lenses(lenses);
        }

        if !lenses.is_empty() && uses_status_command(suggestion.status) {
            return ordered_code_lenses(lenses);
        }

        lenses.push(suggested_code_lens_view(
            dependency,
            Some(&suggestion),
            title,
        ));
        ordered_code_lenses(lenses)
    }
}

fn runtime_constraint_lens(
    dependency: &Dependency,
    suggestion: &Suggestion,
    indicators: &SuggestionIndicators,
) -> LensView {
    let title = if suggestion.status == versionlens_update::SuggestionStatus::SatisfiesLatest {
        format!(
            "checked {}; compatible {}",
            dependency.requirement,
            suggestion.latest.as_deref().unwrap_or("unknown")
        )
    } else {
        code_lens_title(dependency, Some(suggestion), indicators, false)
    };
    LensView::new(dependency.range, title, None)
}

fn ordered_code_lenses(mut lenses: CodeLensViews) -> CodeLensViews {
    for (order, lens) in lenses.iter_mut().enumerate() {
        lens.order = u32::try_from(order).unwrap_or(u32::MAX);
    }
    lenses
}

fn build_has_cached_vulnerabilities(
    session: &VersionLensSession,
    dependency: &Dependency,
    suggestion: &Suggestion,
) -> bool {
    session.has_cached_vulnerabilities(dependency)
        || session.target_update_has_cached_vulnerabilities(Some(suggestion))
}

fn choice_has_cached_vulnerabilities(
    session: &VersionLensSession,
    dependency: &Dependency,
    version: &str,
) -> bool {
    let suggestion = Suggestion {
        dependency: dependency.to_owned(),
        latest: Some(version.to_owned()),
        resolved: None,
        status: StatusUpdateAvailable,
        builds: vec![],
        choices: vec![],
    };
    session.target_update_has_cached_vulnerabilities(Some(&suggestion))
}

fn shows_update_choice_lenses(status: SuggestionStatus) -> bool {
    matches!(
        status,
        StatusCurrent
            | StatusUpdateAvailable
            | StatusInvalid
            | StatusInvalidRange
            | StatusNoMatch
            | StatusFixed
            | StatusSatisfies
            | StatusSatisfiesLatest
    )
}

fn uses_status_command(status: SuggestionStatus) -> bool {
    !matches!(
        status,
        StatusUpdateAvailable | StatusBuildAvailable | StatusDirectory
    )
}

fn build_choice_code_lens_view(
    dependency: &Dependency,
    suggestion: &Suggestion,
    indicators: &SuggestionIndicators,
    has_vulnerabilities: bool,
) -> Option<LensView> {
    if suggestion.builds.is_empty() {
        return None;
    }

    let build_suggestion = Suggestion {
        dependency: dependency.to_owned(),
        latest: suggestion.latest.as_deref().map(|value| value.to_owned()),
        resolved: None,
        status: StatusBuildAvailable,
        builds: suggestion
            .builds
            .iter()
            .map(|value| value.to_owned())
            .collect(),
        choices: vec![],
    };
    let title = code_lens_title(
        dependency,
        Some(&build_suggestion),
        indicators,
        has_vulnerabilities,
    );
    Some(suggested_code_lens_view(
        dependency,
        Some(&build_suggestion),
        title,
    ))
}

fn status_only_code_lens(
    dependency: &Dependency,
    suggestion: Option<&Suggestion>,
    indicators: &SuggestionIndicators,
) -> Option<LensView> {
    let status = status_only_suggestion(dependency, suggestion)?;
    let title = code_lens_title(dependency, Some(&status), indicators, false);
    Some(LensView::new(dependency.range, title, None))
}

fn status_only_suggestion(
    dependency: &Dependency,
    suggestion: Option<&Suggestion>,
) -> Option<Suggestion> {
    let suggestion = suggestion?;
    if suggestion.status == StatusNoMatch {
        return Some(status_suggestion(dependency, None, StatusNoMatch));
    }

    if suggestion.status == StatusNotSupported {
        return Some(status_suggestion(dependency, None, StatusNotSupported));
    }

    let latest = suggestion.latest.as_deref()?;
    if suggestion.status == StatusCurrent {
        return Some(status_suggestion(dependency, Some(latest), StatusCurrent));
    }

    if matches!(
        suggestion.status,
        StatusDirectoryNotFound | StatusInvalid | StatusInvalidRange
    ) {
        return Some(status_suggestion(
            dependency,
            Some(latest),
            suggestion.status,
        ));
    }

    if suggestion.status == StatusSatisfiesLatest {
        return Some(status_suggestion(
            dependency,
            Some(latest),
            StatusSatisfiesLatest,
        ));
    }

    if suggestion.status == StatusSatisfies {
        let version = bump_choice_version(suggestion).unwrap_or(latest);
        return Some(status_suggestion(
            dependency,
            Some(version),
            StatusSatisfies,
        ));
    }

    if suggestion.status == StatusFixed {
        return Some(status_suggestion(dependency, Some(latest), StatusFixed));
    }

    if suggestion.status == StatusError {
        return Some(status_suggestion(dependency, Some(latest), StatusError));
    }

    if suggestion.status != StatusUpdateAvailable
        || !is_fixed_version_requirement(&dependency.requirement, latest)
    {
        return None;
    }

    Some(Suggestion {
        dependency: dependency.to_owned(),
        latest: Some(dependency.requirement.as_str().to_owned()),
        resolved: None,
        status: StatusFixed,
        builds: vec![],
        choices: vec![],
    })
}

fn status_suggestion(
    dependency: &Dependency,
    latest: Option<&str>,
    status: SuggestionStatus,
) -> Suggestion {
    Suggestion {
        dependency: dependency.to_owned(),
        latest: latest.map(|value| value.to_owned()),
        resolved: None,
        status,
        builds: vec![],
        choices: vec![],
    }
}

fn bump_choice_version(suggestion: &Suggestion) -> Option<&str> {
    suggestion
        .choices
        .iter()
        .find(|choice| choice.label.as_str() == "bump")
        .map(|choice| choice.version.as_str())
}

fn is_fixed_version_requirement(requirement: &str, latest: &str) -> bool {
    let requirement = requirement.trim();
    !requirement.is_empty()
        && !requirement.contains([
            '^', '~', '>', '<', '=', '*', '|', ',', '[', ']', '(', ')', ' ',
        ])
        && update_level(latest, requirement).is_some()
}

#[cfg(test)]
mod tests;
