use versionlens_model::Dependency;

use crate::RegistryResponseInput;
use crate::VersionLensSession;
use crate::presentation::view::{DiagnosticView, Severity, UPDATE_AVAILABLE};
use crate::presentation::vulnerability_diagnostics;
use crate::{Suggestion, SuggestionStatus};

impl VersionLensSession {
    pub(crate) fn diagnostics_for_dependency(
        &self,
        dependency: &Dependency,
        responses: &[RegistryResponseInput],
    ) -> Vec<DiagnosticView> {
        if let Some(message) = self.vulnerability_check_failure(dependency) {
            return vec![DiagnosticView::new(
                dependency.requirement_range,
                format!("Vulnerability check failed: {message}"),
                Severity::Information,
                "vulnerability-check-failed".to_owned(),
                None,
            )];
        }
        vulnerability_diagnostics(
            dependency,
            self.vulnerability_advisories(dependency, responses),
        )
    }

    /// The `update-available` diagnostic of a requirement that does not
    /// allow the latest release, or whose floor would move to a new minor or
    /// major. A patch-only floor bump gets none, so ranges that are only a
    /// few patches behind stay out of the problems list.
    pub(crate) fn update_diagnostic(&self, suggestion: &Suggestion) -> Option<DiagnosticView> {
        let severity = self.config.update_severity?;
        let reported = match suggestion.status {
            // `Satisfies` is an update whose range can also take a floor bump.
            SuggestionStatus::UpdateAvailable | SuggestionStatus::Satisfies => true,
            SuggestionStatus::SatisfiesLatest => {
                suggestion.latest.as_deref().is_some_and(|latest| {
                    versionlens_update::floor_crosses_minor(
                        &suggestion.dependency.requirement,
                        latest,
                        versionlens_update::version_dialect(&suggestion.dependency),
                    )
                })
            }
            _ => false,
        };
        if !reported {
            return None;
        }
        let edit = versionlens_update::update_edit(suggestion)?;
        let requirement = suggestion.dependency.requirement.trim();
        (edit.new_text != requirement).then(|| {
            DiagnosticView::new(
                suggestion.dependency.requirement_range,
                format!("{requirement} → {}", edit.new_text),
                severity,
                UPDATE_AVAILABLE.to_owned(),
                None,
            )
        })
    }
}
