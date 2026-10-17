use crate::cargo_toml::parse_cargo_toml_project_versions;
use crate::classify::classify_document;
use crate::json_manifest::parse_package_json_project_version;
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
    match kind {
        ManifestKind::NpmPackageJson | ManifestKind::NpmPackageJson5 => {
            parse_package_json_project_version(&input.text)
        }
        ManifestKind::CargoToml => parse_cargo_toml_project_versions(&input.text),
        _ => Vec::new(),
    }
}
