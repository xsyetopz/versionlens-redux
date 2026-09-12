use versionlens_model::Dependency;
use versionlens_vscode_model::DiagnosticPayload;

use crate::RegistryResponseInput;
use crate::VersionLensSession;
use crate::presentation::vulnerability_diagnostics;

impl VersionLensSession {
    pub(crate) fn diagnostics_for_dependency(
        &self,
        dependency: &Dependency,
        responses: &[RegistryResponseInput],
    ) -> Vec<DiagnosticPayload> {
        if let Some(message) = self.vulnerability_check_failure(dependency) {
            return vec![DiagnosticPayload {
                range: dependency.requirement_range,
                message: format!("Vulnerability check failed: {message}"),
                severity: 2,
                source: Some("VersionLens".to_owned()),
                code: Some("vulnerability-check-failed".to_owned()),
                code_description_url: None,
            }];
        }
        vulnerability_diagnostics(
            dependency,
            self.vulnerability_advisories(dependency, responses),
        )
    }
}
