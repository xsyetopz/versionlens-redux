use versionlens_model::Dependency;
use versionlens_model::Range;
use versionlens_suggestions::SuggestionStatus::{
    BuildAvailable as StatusBuildAvailable, Current as StatusCurrent, Directory as StatusDirectory,
    DirectoryNotFound as StatusDirectoryNotFound, Error as StatusError, Fixed as StatusFixed,
    Invalid as StatusInvalid, InvalidRange as StatusInvalidRange, NoMatch as StatusNoMatch,
    NotSupported as StatusNotSupported, Satisfies as StatusSatisfies,
    SatisfiesLatest as StatusSatisfiesLatest, UpdateAvailable as StatusUpdateAvailable,
};
use versionlens_suggestions::{Suggestion, SuggestionStatus};
use versionlens_versions::update_level;
use versionlens_vscode_model::CodeLensPayload;

use crate::presentation::{
    code_lens_title, project_version_code_lens_payload, suggested_code_lens_payload,
    update_choice_code_lens_payload,
};
use crate::project::project_version_code_lens_suggestions;
use crate::{SuggestionIndicators, VersionLensSession};
use versionlens_model::Ecosystem::Docker;

type CodeLensPayloads = Vec<CodeLensPayload>;

impl VersionLensSession {
    pub(crate) fn code_lenses_for_dependency(&self, dependency: &Dependency) -> CodeLensPayloads {
        let project_version_suggestions = project_version_code_lens_suggestions(dependency);
        if !project_version_suggestions.is_empty() {
            return ordered_code_lenses(
                project_version_suggestions
                    .into_iter()
                    .map(|suggestion| {
                        project_version_code_lens_payload(
                            dependency,
                            suggestion,
                            &self.config.suggestion_indicators,
                        )
                    })
                    .collect(),
            );
        }

        let Some(suggestion) = self.cached_suggestion(dependency) else {
            return vec![];
        };

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
            if let Some(build_lens) = build_choice_code_lens_payload(
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
                update_choice_code_lens_payload(
                    dependency,
                    choice,
                    &self.config.suggestion_indicators,
                    choice_has_cached_vulnerabilities(self, dependency, choice.version.as_str()),
                )
            }));
            if let Some(build_lens) = build_choice_code_lens_payload(
                dependency,
                &suggestion,
                &self.config.suggestion_indicators,
                build_has_cached_vulnerabilities(self, dependency, &suggestion),
            ) {
                lenses.push(build_lens);
            }
            return ordered_code_lenses(lenses);
        }

        if let Some(build_lens) = build_choice_code_lens_payload(
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

        lenses.push(suggested_code_lens_payload(
            dependency,
            Some(&suggestion),
            title,
        ));
        ordered_code_lenses(lenses)
    }
}

fn ordered_code_lenses(mut lenses: CodeLensPayloads) -> CodeLensPayloads {
    for (order, lens) in lenses.iter_mut().enumerate() {
        lens.range = ordered_code_lens_range(lens.range, order);
    }
    lenses
}

fn ordered_code_lens_range(range: Range, order: usize) -> Range {
    let Ok(offset) = u32::try_from(order) else {
        return range;
    };
    let mut ordered = range;
    ordered.start.character = ordered
        .start
        .character
        .saturating_add(offset)
        .min(range.end.character);
    ordered.end = ordered.start;
    ordered
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

fn build_choice_code_lens_payload(
    dependency: &Dependency,
    suggestion: &Suggestion,
    indicators: &SuggestionIndicators,
    has_vulnerabilities: bool,
) -> Option<CodeLensPayload> {
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
    Some(suggested_code_lens_payload(
        dependency,
        Some(&build_suggestion),
        title,
    ))
}

fn status_only_code_lens(
    dependency: &Dependency,
    suggestion: Option<&Suggestion>,
    indicators: &SuggestionIndicators,
) -> Option<CodeLensPayload> {
    let status = status_only_suggestion(dependency, suggestion)?;
    let title = code_lens_title(dependency, Some(&status), indicators, false);
    Some(CodeLensPayload {
        range: dependency.range,
        title,
        command: "".to_owned(),
        arguments: vec![],
    })
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
