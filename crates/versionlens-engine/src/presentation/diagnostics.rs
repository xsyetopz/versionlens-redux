use std::borrow::Cow::{Borrowed as CowBorrowed, Owned as CowOwned};

use versionlens_model::Dependency;
use versionlens_providers::{VulnerabilityAdvisory, vulnerability_version_from_requirement};

use crate::view::{DiagnosticView, Severity};

const DIAGNOSTIC_SOURCE: &str = "VersionLens";

pub(crate) fn vulnerability_diagnostics(
    dependency: &Dependency,
    advisories: Vec<VulnerabilityAdvisory>,
) -> Vec<DiagnosticView> {
    let version = vulnerability_version_from_requirement(&dependency.requirement)
        .map(CowOwned)
        .unwrap_or(CowBorrowed(&dependency.requirement));

    advisories
        .into_iter()
        .map(|advisory| DiagnosticView {
            range: dependency.requirement_range,
            message: format!(
                "Vulnerability found in {}@{}:\n{}",
                dependency.name, version, advisory.title
            ),
            severity: Severity::Error,
            source: Some(DIAGNOSTIC_SOURCE.to_owned()),
            code: Some(advisory.id),
            code_description_url: advisory.url,
        })
        .collect()
}
