use crate::model::{DocumentInput, ManifestKind};

mod content;
mod patterns;
mod tables;
mod uri;

use content::classify_content_manifest;
use patterns::{classify_early_pattern_manifest, classify_python_manifest, is_dockerfile_uri};
use tables::{
    EARLY_FILE_MANIFESTS, LATE_FILE_MANIFESTS, PUBSPEC_FILE_MANIFESTS, exact_file_manifest,
};
use uri::{SCHEMA_URI, document_uri, is_file_uri};

pub fn classify_document(input: &DocumentInput) -> ManifestKind {
    let uri = document_uri(&input.uri);
    if let Some(kind) = classify_special_uri(uri) {
        return kind;
    }
    if !is_file_uri(uri) {
        return ManifestKind::Unknown;
    }

    classify_early_manifest(uri)
        .or_else(|| classify_docker_manifest(&input.language_id, uri))
        .or_else(|| exact_file_manifest(uri, LATE_FILE_MANIFESTS))
        .or_else(|| classify_content_manifest(input, uri))
        .or_else(|| classify_python_manifest(&input.language_id, uri))
        .or_else(|| exact_file_manifest(uri, PUBSPEC_FILE_MANIFESTS))
        .unwrap_or(ManifestKind::Unknown)
}

fn classify_special_uri(uri: &str) -> Option<ManifestKind> {
    (uri == SCHEMA_URI).then_some(ManifestKind::VersionLensMultiRegistries)
}

fn classify_early_manifest(uri: &str) -> Option<ManifestKind> {
    exact_file_manifest(uri, EARLY_FILE_MANIFESTS).or_else(|| classify_early_pattern_manifest(uri))
}

fn classify_docker_manifest(language_id: &str, uri: &str) -> Option<ManifestKind> {
    if language_id == "dockercompose" {
        return Some(ManifestKind::DockerComposeYaml);
    }
    (language_id == "dockerfile" || is_dockerfile_uri(uri)).then_some(ManifestKind::Dockerfile)
}

#[cfg(test)]
mod tests;
