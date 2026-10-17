use crate::manifest::manifest_row;
use versionlens_model::{Dependency, ManifestKind};

pub(super) fn parse_manifest_kind(
    kind: ManifestKind,
    text: &str,
    paths: &[&str],
) -> Vec<Dependency> {
    manifest_row(kind).map_or_else(Vec::new, |row| row.parser.parse(text, paths))
}
