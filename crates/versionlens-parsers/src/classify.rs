use versionlens_model::ManifestKind::{Unknown, VersionLensMultiRegistries};
use versionlens_model::{DocumentInput, ManifestKind};

use crate::manifest::{MANIFESTS, Stage};

mod uri;

use uri::{SCHEMA_URI, document_uri, is_file_uri};

pub fn classify_document(input: &DocumentInput) -> ManifestKind {
    let uri = document_uri(&input.uri);
    if uri == SCHEMA_URI {
        return VersionLensMultiRegistries;
    }
    if !is_file_uri(uri) {
        return Unknown;
    }

    Stage::ALL
        .iter()
        .find_map(|&stage| {
            MANIFESTS.iter().find_map(|row| {
                row.patterns
                    .iter()
                    .any(|pattern| pattern.stage == stage && pattern.rule.matches(input, uri))
                    .then_some(row.kind)
            })
        })
        .unwrap_or(Unknown)
}

#[cfg(test)]
mod tests;
