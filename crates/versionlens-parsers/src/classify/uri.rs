use std::path::Path;

pub(super) const SCHEMA_URI: &str = "versionlens:/versionlens.multi-registries.json";

pub(super) fn document_uri(uri: &str) -> &str {
    uri.split(['?', '#']).next().unwrap_or(uri)
}

pub(super) fn is_file_uri(uri: &str) -> bool {
    uri.starts_with("file:")
}

pub(super) fn file_name(uri: &str) -> Option<&str> {
    uri.rsplit('/').next()
}

pub(super) fn has_extension<const N: usize>(name: &str, extensions: [&str; N]) -> bool {
    Path::new(name).extension().is_some_and(|extension| {
        extensions
            .iter()
            .any(|item| extension.eq_ignore_ascii_case(item))
    })
}
