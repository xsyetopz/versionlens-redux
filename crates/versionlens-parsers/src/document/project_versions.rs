use crate::classify::classify_document;
use crate::manifest::manifest_row;
use versionlens_model::{Dependency, DocumentInput, ManifestKind};

/// The project version fields of a document: the npm `version` and the Cargo
/// package version. They are read apart from `parse_document`, so dependency
/// listings never carry them.
pub fn parse_project_versions(input: &DocumentInput) -> Vec<Dependency> {
    parse_project_versions_as_manifest(input, classify_document(input))
}

pub fn parse_project_versions_as_manifest(
    input: &DocumentInput,
    kind: ManifestKind,
) -> Vec<Dependency> {
    manifest_row(kind).map_or_else(Vec::new, |row| row.parse_own_version(&input.text))
}
