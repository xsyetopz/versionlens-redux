use versionlens_model::Ecosystem;
use versionlens_model::ManifestKind::{
    BunVersion, CargoToml, GoMod, NodeVersion, Nvmrc, RustToolchain, RustToolchainToml, Unknown,
    VersionLensMultiRegistries,
};

use super::{
    MANIFESTS, ManifestFormat, ecosystem_for_manifest, manifest_row, provider_name_for_manifest,
};

#[test]
fn lists_each_manifest_kind_once_with_a_pattern() {
    for (index, row) in MANIFESTS.iter().enumerate() {
        assert!(!row.patterns.is_empty(), "{:?} has no pattern", row.kind);
        assert!(
            MANIFESTS[index + 1..]
                .iter()
                .all(|other| other.kind != row.kind),
            "{:?} has two rows",
            row.kind
        );
    }
}

#[test]
fn maps_manifest_kinds_to_their_row() {
    for row in MANIFESTS {
        assert_eq!(ecosystem_for_manifest(row.kind), Some(row.ecosystem));
        assert_eq!(provider_name_for_manifest(row.kind), Some(row.provider));
    }
    assert_eq!(ecosystem_for_manifest(GoMod), Some(Ecosystem::Go));
    assert_eq!(provider_name_for_manifest(GoMod), Some("golang"));
    assert_eq!(
        manifest_row(CargoToml).map(|row| row.format),
        Some(ManifestFormat::Toml)
    );
}

#[test]
fn ignores_non_dependency_manifest_kinds() {
    for kind in [Unknown, VersionLensMultiRegistries] {
        assert!(manifest_row(kind).is_none());
        assert_eq!(ecosystem_for_manifest(kind), None);
        assert_eq!(provider_name_for_manifest(kind), None);
    }
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

#[test]
fn rows_hold_their_default_dependency_paths() {
    use versionlens_model::ManifestKind::{NixFlake, NpmPackageJson, PubspecOverridesYaml};

    let defaults = |kind| manifest_row(kind).expect("row exists").dependency_paths;

    let npm = defaults(NpmPackageJson);
    assert!(npm.contains(&"dependencies"));
    assert!(!npm.contains(&"version"));
    assert_eq!(defaults(CargoToml).first(), Some(&"package"));
    assert_eq!(defaults(PubspecOverridesYaml), ["dependency_overrides"]);
    assert!(defaults(NixFlake).is_empty());
    assert!(defaults(GoMod).is_empty());
}
