use super::{
    Ecosystem, ManifestKind, ecosystem_config_namespace, ecosystem_for_manifest,
    ecosystem_from_config_name,
};

#[test]
fn maps_config_names_and_legacy_names_to_ecosystems() {
    let cases = [
        ("cargo", Ecosystem::Cargo),
        ("composer", Ecosystem::Composer),
        ("deno", Ecosystem::Deno),
        ("dotnet", Ecosystem::Dotnet),
        ("docker", Ecosystem::Docker),
        ("dub", Ecosystem::Dub),
        ("go", Ecosystem::Go),
        ("golang", Ecosystem::Go),
        ("maven", Ecosystem::Maven),
        ("bun", Ecosystem::Npm),
        ("npm", Ecosystem::Npm),
        ("pnpm", Ecosystem::Npm),
        ("pypi", Ecosystem::Python),
        ("python", Ecosystem::Python),
        ("pub", Ecosystem::Pub),
        ("ruby", Ecosystem::Ruby),
    ];

    for (name, ecosystem) in cases {
        assert_eq!(ecosystem_from_config_name(name), Some(ecosystem));
    }
}

#[test]
fn ignores_unknown_config_names() {
    assert_eq!(ecosystem_from_config_name("unknown"), None);
}

#[test]
fn maps_ecosystems_to_config_namespaces() {
    let cases = [
        (Ecosystem::Cargo, "cargo"),
        (Ecosystem::Composer, "composer"),
        (Ecosystem::Deno, "deno"),
        (Ecosystem::Dotnet, "dotnet"),
        (Ecosystem::Docker, "docker"),
        (Ecosystem::Dub, "dub"),
        (Ecosystem::Go, "golang"),
        (Ecosystem::Maven, "maven"),
        (Ecosystem::Npm, "npm"),
        (Ecosystem::Python, "pypi"),
        (Ecosystem::Pub, "pub"),
        (Ecosystem::Ruby, "ruby"),
    ];

    for (ecosystem, namespace) in cases {
        assert_eq!(ecosystem_config_namespace(ecosystem), namespace);
    }
}

#[test]
fn maps_manifest_kinds_to_ecosystems() {
    let cases = [
        (ManifestKind::CargoToml, Ecosystem::Cargo),
        (ManifestKind::ComposerJson, Ecosystem::Composer),
        (ManifestKind::DenoJson, Ecosystem::Deno),
        (ManifestKind::DotnetProjectJson, Ecosystem::Dotnet),
        (ManifestKind::DotnetXml, Ecosystem::Dotnet),
        (ManifestKind::DockerComposeYaml, Ecosystem::Docker),
        (ManifestKind::Dockerfile, Ecosystem::Docker),
        (ManifestKind::DubJson, Ecosystem::Dub),
        (ManifestKind::Gemfile, Ecosystem::Ruby),
        (ManifestKind::GoMod, Ecosystem::Go),
        (ManifestKind::MavenPomXml, Ecosystem::Maven),
        (ManifestKind::NpmPackageJson, Ecosystem::Npm),
        (ManifestKind::PnpmYaml, Ecosystem::Npm),
        (ManifestKind::PythonPipfile, Ecosystem::Python),
        (ManifestKind::PythonPyprojectToml, Ecosystem::Python),
        (ManifestKind::PythonRequirementsTxt, Ecosystem::Python),
        (ManifestKind::PubspecYaml, Ecosystem::Pub),
    ];

    for (kind, ecosystem) in cases {
        assert_eq!(ecosystem_for_manifest(kind), Some(ecosystem));
    }
}

#[test]
fn ignores_non_dependency_manifest_kinds() {
    assert_eq!(ecosystem_for_manifest(ManifestKind::Unknown), None);
    assert_eq!(
        ecosystem_for_manifest(ManifestKind::VersionLensMultiRegistries),
        None
    );
}
