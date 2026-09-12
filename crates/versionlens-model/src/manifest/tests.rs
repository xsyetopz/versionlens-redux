use super::{MANIFEST_ECOSYSTEMS, ecosystem_for_manifest};
use crate::ManifestKind::{
    BunVersion, NodeVersion, Nvmrc, RustToolchain, RustToolchainToml, Unknown,
    VersionLensMultiRegistries,
};
use crate::provider_name_for_manifest;

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

#[test]
fn maps_runtime_manifests_to_their_provider_families() {
    for kind in [Nvmrc, NodeVersion, BunVersion] {
        assert_eq!(provider_name_for_manifest(kind), Some("npm"));
    }
    for kind in [RustToolchain, RustToolchainToml] {
        assert_eq!(provider_name_for_manifest(kind), Some("cargo"));
    }
}
