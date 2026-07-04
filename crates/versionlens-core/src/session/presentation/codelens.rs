use versionlens_parsers::{Dependency, Ecosystem};
use versionlens_suggestions::{Suggestion, SuggestionStatus};
use versionlens_versions::update_level;
use versionlens_vscode_model::CodeLensPayload;

use crate::presentation::{
    code_lens_title, project_version_code_lens_payload, suggested_code_lens_payload,
    update_choice_code_lens_payload,
};
use crate::project::project_version_code_lens_suggestions;
use crate::{SuggestionIndicators, VersionLensSession};

impl VersionLensSession {
    pub(crate) fn code_lenses_for_dependency(
        &self,
        dependency: &Dependency,
    ) -> Vec<CodeLensPayload> {
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
            return Vec::new();
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

        if suggestion.status == SuggestionStatus::Current
            && dependency.ecosystem != Ecosystem::Docker
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

        if shows_update_choice_lenses(dependency, suggestion.status)
            && !suggestion.choices.is_empty()
        {
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

fn ordered_code_lenses(mut lenses: Vec<CodeLensPayload>) -> Vec<CodeLensPayload> {
    for (order, lens) in lenses.iter_mut().enumerate() {
        lens.range = ordered_code_lens_range(lens.range, order);
    }
    lenses
}

fn ordered_code_lens_range(
    range: versionlens_vscode_model::Range,
    order: usize,
) -> versionlens_vscode_model::Range {
    let Ok(offset) = u32::try_from(order) else {
        return range;
    };
    let mut ordered = range;
    ordered.start.character = ordered.start.character.saturating_add(offset);
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
        status: SuggestionStatus::UpdateAvailable,
        builds: Vec::new(),
        choices: Vec::new(),
    };
    session.target_update_has_cached_vulnerabilities(Some(&suggestion))
}

fn shows_update_choice_lenses(dependency: &Dependency, status: SuggestionStatus) -> bool {
    if status == SuggestionStatus::Current {
        return dependency.ecosystem == Ecosystem::Docker;
    }

    matches!(
        status,
        SuggestionStatus::UpdateAvailable
            | SuggestionStatus::Invalid
            | SuggestionStatus::InvalidRange
            | SuggestionStatus::NoMatch
            | SuggestionStatus::Fixed
            | SuggestionStatus::Satisfies
            | SuggestionStatus::SatisfiesLatest
    )
}

fn uses_status_command(status: SuggestionStatus) -> bool {
    !matches!(
        status,
        SuggestionStatus::UpdateAvailable
            | SuggestionStatus::BuildAvailable
            | SuggestionStatus::Directory
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
        latest: suggestion.latest.as_deref().map(str::to_owned),
        resolved: None,
        status: SuggestionStatus::BuildAvailable,
        builds: suggestion.builds.iter().map(String::to_owned).collect(),
        choices: Vec::new(),
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
        command: String::new(),
        arguments: Vec::new(),
    })
}

fn status_only_suggestion(
    dependency: &Dependency,
    suggestion: Option<&Suggestion>,
) -> Option<Suggestion> {
    let suggestion = suggestion?;
    if suggestion.status == SuggestionStatus::NoMatch {
        return Some(status_suggestion(
            dependency,
            None,
            SuggestionStatus::NoMatch,
        ));
    }

    if suggestion.status == SuggestionStatus::NotSupported {
        return Some(status_suggestion(
            dependency,
            None,
            SuggestionStatus::NotSupported,
        ));
    }

    let latest = suggestion.latest.as_deref()?;
    if suggestion.status == SuggestionStatus::Current {
        return Some(status_suggestion(
            dependency,
            Some(latest),
            SuggestionStatus::Current,
        ));
    }

    if matches!(
        suggestion.status,
        SuggestionStatus::DirectoryNotFound
            | SuggestionStatus::Invalid
            | SuggestionStatus::InvalidRange
    ) {
        return Some(status_suggestion(
            dependency,
            Some(latest),
            suggestion.status,
        ));
    }

    if suggestion.status == SuggestionStatus::SatisfiesLatest {
        return Some(status_suggestion(
            dependency,
            Some(latest),
            SuggestionStatus::SatisfiesLatest,
        ));
    }

    if suggestion.status == SuggestionStatus::Satisfies {
        let version = bump_choice_version(suggestion).unwrap_or(latest);
        return Some(status_suggestion(
            dependency,
            Some(version),
            SuggestionStatus::Satisfies,
        ));
    }

    if suggestion.status == SuggestionStatus::Fixed {
        return Some(status_suggestion(
            dependency,
            Some(latest),
            SuggestionStatus::Fixed,
        ));
    }

    if suggestion.status == SuggestionStatus::Error {
        return Some(status_suggestion(
            dependency,
            Some(latest),
            SuggestionStatus::Error,
        ));
    }

    if suggestion.status != SuggestionStatus::UpdateAvailable
        || !is_fixed_version_requirement(&dependency.requirement, latest)
    {
        return None;
    }

    Some(Suggestion {
        dependency: dependency.to_owned(),
        latest: Some(dependency.requirement.as_str().to_owned()),
        resolved: None,
        status: SuggestionStatus::Fixed,
        builds: Vec::new(),
        choices: Vec::new(),
    })
}

fn status_suggestion(
    dependency: &Dependency,
    latest: Option<&str>,
    status: SuggestionStatus,
) -> Suggestion {
    Suggestion {
        dependency: dependency.to_owned(),
        latest: latest.map(str::to_owned),
        resolved: None,
        status,
        builds: Vec::new(),
        choices: Vec::new(),
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
