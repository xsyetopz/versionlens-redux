use super::super::uri::{file_name, has_extension};
use versionlens_model::ManifestKind;
use versionlens_model::ManifestKind::{Cmake, MesonWrap};

pub(super) fn classify_cpp_manifest(uri: &str) -> Option<ManifestKind> {
    let name = file_name(uri)?;
    if has_extension(name, ["cmake"]) {
        return Some(Cmake);
    }
    if has_extension(name, ["wrap"]) {
        return Some(MesonWrap);
    }
    None
}
