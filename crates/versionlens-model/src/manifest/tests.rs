use super::{MANIFEST_ECOSYSTEMS, ecosystem_for_manifest};
use crate::ManifestKind::{Unknown, VersionLensMultiRegistries};

#[test]
fn maps_manifest_kinds_to_ecosystems() {
    for &(kind, ecosystem) in MANIFEST_ECOSYSTEMS {
        assert_eq!(ecosystem_for_manifest(kind), Some(ecosystem));
    }
}

#[test]
fn ignores_non_dependency_manifest_kinds() {
    assert_eq!(ecosystem_for_manifest(Unknown), None);
    assert_eq!(ecosystem_for_manifest(VersionLensMultiRegistries), None);
}
