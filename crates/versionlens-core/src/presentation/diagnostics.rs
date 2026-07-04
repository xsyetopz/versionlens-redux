use std::borrow::Cow;

use versionlens_parsers::Dependency;
use versionlens_providers::{VulnerabilityAdvisory, vulnerability_version_from_requirement};
use versionlens_vscode_model::DiagnosticPayload;

const ERROR_SEVERITY: u8 = 0;
const DIAGNOSTIC_SOURCE: &str = "VersionLens";

pub(crate) fn vulnerability_diagnostics(
    dependency: &Dependency,
    advisories: Vec<VulnerabilityAdvisory>,
) -> Vec<DiagnosticPayload> {
    let version = vulnerability_version_from_requirement(&dependency.requirement)
        .map(Cow::Owned)
        .unwrap_or(Cow::Borrowed(&dependency.requirement));

    advisories
        .into_iter()
        .map(|advisory| DiagnosticPayload {
            range: dependency.requirement_range,
            message: format!(
                "Vulnerability found in {}@{}:\n{}",
                dependency.name, version, advisory.title
            ),
            severity: ERROR_SEVERITY,
            source: Some(DIAGNOSTIC_SOURCE.to_owned()),
            code: Some(advisory.id),
            code_description_url: advisory.url,
        })
        .collect()
}
