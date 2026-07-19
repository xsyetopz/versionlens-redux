use crate::json_manifest::looks_like_package_json;
use versionlens_model::{DocumentInput, ManifestKind};

use super::uri::{file_name, has_extension};
use versionlens_model::ManifestKind::NpmPackageJson;

pub(super) fn classify_content_manifest(input: &DocumentInput, uri: &str) -> Option<ManifestKind> {
    if is_json_document(&input.language_id, uri) && looks_like_package_json(&input.text) {
        return Some(NpmPackageJson);
    }
    None
}

fn is_json_document(language_id: &str, uri: &str) -> bool {
    let name = file_name(uri).unwrap_or(uri);
    matches!(language_id, "json" | "jsonc") && has_extension(name, ["json", "jsonc"])
}
