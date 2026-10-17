pub(super) const SCHEMA_URI: &str = "versionlens:/multi-registries.json";

pub(super) fn document_uri(uri: &str) -> &str {
    uri.split(['?', '#']).next().unwrap_or(uri)
}

pub(super) fn is_file_uri(uri: &str) -> bool {
    uri.starts_with("file:")
}
