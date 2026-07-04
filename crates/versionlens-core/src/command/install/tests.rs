use versionlens_parsers::{Ecosystem, ManifestKind};

use super::{install_task_config_key, install_task_config_key_for_manifest};

#[test]
fn install_task_config_keys_match_provider_settings() {
    let cases = [
        (Ecosystem::Cargo, "cargo.onSaveChanges"),
        (Ecosystem::Composer, "composer.onSaveChanges"),
        (Ecosystem::Deno, "deno.onSaveChanges"),
        (Ecosystem::Dotnet, "dotnet.onSaveChanges"),
        (Ecosystem::Dub, "dub.onSaveChanges"),
        (Ecosystem::Go, "golang.onSaveChanges"),
        (Ecosystem::Maven, "maven.onSaveChanges"),
        (Ecosystem::Npm, "npm.onSaveChanges"),
        (Ecosystem::Python, "pypi.onSaveChanges"),
        (Ecosystem::Pub, "pub.onSaveChanges"),
        (Ecosystem::Ruby, "ruby.onSaveChanges"),
    ];

    for (ecosystem, key) in cases {
        assert_eq!(install_task_config_key(ecosystem), key);
    }
}

#[test]
fn install_task_keys_follow_smoke_provider_capabilities() {
    assert_eq!(
        install_task_config_key_for_manifest(ManifestKind::NpmPackageJson),
        Some("npm.onSaveChanges".to_owned())
    );
    assert_eq!(
        install_task_config_key_for_manifest(ManifestKind::DenoJson),
        Some("deno.onSaveChanges".to_owned())
    );
    assert_eq!(
        install_task_config_key_for_manifest(ManifestKind::PnpmYaml),
        None
    );
    assert_eq!(
        install_task_config_key_for_manifest(ManifestKind::Dockerfile),
        None
    );
    assert_eq!(
        install_task_config_key_for_manifest(ManifestKind::DockerComposeYaml),
        None
    );
}
