use super::{install_task_config_key, install_task_config_key_for_manifest};
use versionlens_parsers::Ecosystem::{
    Cargo, Composer, Deno, Dotnet, Dub, Go, Maven, Npm, Pub, Python, Ruby,
};
use versionlens_parsers::ManifestKind::{
    DenoJson, DockerComposeYaml, Dockerfile, NpmPackageJson, PnpmYaml,
};

#[test]
fn install_task_config_keys_match_provider_settings() {
    let cases = [
        (Cargo, "cargo.onSaveChanges"),
        (Composer, "composer.onSaveChanges"),
        (Deno, "deno.onSaveChanges"),
        (Dotnet, "dotnet.onSaveChanges"),
        (Dub, "dub.onSaveChanges"),
        (Go, "golang.onSaveChanges"),
        (Maven, "maven.onSaveChanges"),
        (Npm, "npm.onSaveChanges"),
        (Python, "pypi.onSaveChanges"),
        (Pub, "pub.onSaveChanges"),
        (Ruby, "ruby.onSaveChanges"),
    ];

    for (ecosystem, key) in cases {
        assert_eq!(install_task_config_key(ecosystem), key);
    }
}

#[test]
fn install_task_keys_follow_smoke_provider_capabilities() {
    assert_eq!(
        install_task_config_key_for_manifest(NpmPackageJson),
        Some("npm.onSaveChanges".to_owned())
    );
    assert_eq!(
        install_task_config_key_for_manifest(DenoJson),
        Some("deno.onSaveChanges".to_owned())
    );
    assert_eq!(install_task_config_key_for_manifest(PnpmYaml), None);
    assert_eq!(install_task_config_key_for_manifest(Dockerfile), None);
    assert_eq!(
        install_task_config_key_for_manifest(DockerComposeYaml),
        None
    );
}
