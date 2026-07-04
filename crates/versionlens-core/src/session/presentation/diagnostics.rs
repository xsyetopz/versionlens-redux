use versionlens_parsers::Dependency;
use versionlens_vscode_model::DiagnosticPayload;

use crate::VersionLensSession;
use crate::model::RegistryResponseInput;
use crate::presentation::vulnerability_diagnostics;

impl VersionLensSession {
    pub(crate) fn diagnostics_for_dependency(
        &self,
        dependency: &Dependency,
        responses: &[RegistryResponseInput],
    ) -> Vec<DiagnosticPayload> {
        vulnerability_diagnostics(
            dependency,
            self.vulnerability_advisories(dependency, responses),
        )
    }
}
