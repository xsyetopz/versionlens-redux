use crate::model::{DocumentInput, ManifestKind};

use super::classify_document;

#[test]
fn classifies_supported_json_toml_and_xml_manifest_files() {
    for (uri, language_id, kind) in [
        (
            "file:///work/package.json",
            "jsonc",
            ManifestKind::NpmPackageJson,
        ),
        ("file:///work/Cargo.toml", "toml", ManifestKind::CargoToml),
        (
            "file:///work/composer.json",
            "json",
            ManifestKind::ComposerJson,
        ),
        ("file:///work/deno.json", "jsonc", ManifestKind::DenoJson),
        ("file:///work/deno.jsonc", "jsonc", ManifestKind::DenoJson),
        (
            "file:///work/project.json",
            "json",
            ManifestKind::DotnetProjectJson,
        ),
        ("file:///work/app.csproj", "xml", ManifestKind::DotnetXml),
        ("file:///work/app.fsproj", "xml", ManifestKind::DotnetXml),
        ("file:///work/app.vbproj", "xml", ManifestKind::DotnetXml),
        (
            "file:///work/Directory.Packages.props",
            "xml",
            ManifestKind::DotnetXml,
        ),
        (
            "file:///work/Directory.Build.targets",
            "xml",
            ManifestKind::DotnetXml,
        ),
        ("file:///work/dub.json", "json", ManifestKind::DubJson),
        ("file:///work/Pipfile", "toml", ManifestKind::PythonPipfile),
        (
            "file:///work/pyproject.toml",
            "toml",
            ManifestKind::PythonPyprojectToml,
        ),
        ("file:///work/pom.xml", "xml", ManifestKind::MavenPomXml),
    ] {
        assert_manifest(uri, language_id, kind);
    }
}

#[test]
fn classifies_supported_yaml_plaintext_and_other_manifest_files() {
    for (uri, language_id, kind) in [
        (
            "file:///work/Dockerfile",
            "dockerfile",
            ManifestKind::Dockerfile,
        ),
        (
            "file:///work/build.Dockerfile",
            "dockerfile",
            ManifestKind::Dockerfile,
        ),
        (
            "file:///work/compose.yaml",
            "yaml",
            ManifestKind::DockerComposeYaml,
        ),
        (
            "file:///work/docker-compose.yaml",
            "yaml",
            ManifestKind::DockerComposeYaml,
        ),
        (
            "file:///work/docker-compose.override.yml",
            "dockercompose",
            ManifestKind::DockerComposeYaml,
        ),
        (
            "file:///work/docker-compose.prod.yaml",
            "yaml",
            ManifestKind::DockerComposeYaml,
        ),
        (
            "file:///work/compose.override.yaml",
            "yaml",
            ManifestKind::DockerComposeYaml,
        ),
        ("file:///work/Gemfile", "ruby", ManifestKind::Gemfile),
        ("file:///work/go.mod", "go.mod", ManifestKind::GoMod),
        (
            "file:///work/requirements-dev.txt",
            "plaintext",
            ManifestKind::PythonRequirementsTxt,
        ),
        (
            "file:///work/pubspec.yaml",
            "yaml",
            ManifestKind::PubspecYaml,
        ),
        (
            "file:///work/pubspec.yml",
            "yaml",
            ManifestKind::PubspecYaml,
        ),
        (
            "file:///work/pnpm-workspace.yaml",
            "yaml",
            ManifestKind::PnpmYaml,
        ),
        (
            "file:///work/pnpm-workspace.yml",
            "yaml",
            ManifestKind::PnpmYaml,
        ),
        ("file:///work/.yarnrc.yaml", "yaml", ManifestKind::PnpmYaml),
        ("file:///work/.yarnrc.yml", "yaml", ManifestKind::PnpmYaml),
        (
            "file:///work/service.compose.yml",
            "yaml",
            ManifestKind::DockerComposeYaml,
        ),
        (
            "file:///work/build.dockerfile",
            "dockerfile",
            ManifestKind::Dockerfile,
        ),
        (
            "versionlens:/versionlens.multi-registries.json",
            "json",
            ManifestKind::VersionLensMultiRegistries,
        ),
    ] {
        assert_manifest(uri, language_id, kind);
    }
}

#[test]
fn classifies_known_manifest_paths_without_language_ids() {
    let cases = [
        ("file:///work/Cargo.toml", ManifestKind::CargoToml),
        ("file:///work/composer.json", ManifestKind::ComposerJson),
        ("file:///work/deno.jsonc", ManifestKind::DenoJson),
        ("file:///work/project.csproj", ManifestKind::DotnetXml),
        ("file:///work/project.vbproj", ManifestKind::DotnetXml),
        ("file:///work/pom.xml", ManifestKind::MavenPomXml),
        (
            "file:///work/docker-compose.yaml",
            ManifestKind::DockerComposeYaml,
        ),
        (
            "file:///work/docker-compose.override.yml",
            ManifestKind::DockerComposeYaml,
        ),
        (
            "file:///work/compose.override.yaml",
            ManifestKind::DockerComposeYaml,
        ),
        ("file:///work/pnpm-workspace.yaml", ManifestKind::PnpmYaml),
        ("file:///work/dub.selections.json", ManifestKind::DubJson),
        ("file:///work/Gemfile", ManifestKind::Gemfile),
        ("file:///work/go.mod", ManifestKind::GoMod),
        ("file:///work/package.json", ManifestKind::NpmPackageJson),
        ("file:///work/Pipfile", ManifestKind::PythonPipfile),
        (
            "file:///work/pyproject.toml",
            ManifestKind::PythonPyprojectToml,
        ),
        ("file:///work/pubspec.yaml", ManifestKind::PubspecYaml),
    ];

    for (uri, expected) in cases {
        assert_eq!(
            classify_document(&DocumentInput {
                uri: uri.to_owned(),
                language_id: "plaintext".to_owned(),
                text: "{}".to_owned(),
                workspace_root: None,
            }),
            expected,
        );
    }
}

#[test]
fn ignores_ordinary_manifests_from_non_file_uris() {
    for uri in [
        "untitled:/package.json",
        "git:/work/package.json",
        "vscode-notebook-cell:/work/Cargo.toml",
    ] {
        assert_eq!(
            classify_document(&DocumentInput {
                uri: uri.to_owned(),
                language_id: "json".to_owned(),
                text: r#"{"dependencies":{"left-pad":"1.0.0"}}"#.to_owned(),
                workspace_root: None,
            }),
            ManifestKind::Unknown,
        );
    }

    assert_eq!(
        classify_document(&DocumentInput {
            uri: "versionlens:/versionlens.multi-registries.json".to_owned(),
            language_id: "json".to_owned(),
            text: String::new(),
            workspace_root: None,
        }),
        ManifestKind::VersionLensMultiRegistries,
    );
}

fn assert_manifest(uri: &str, language_id: &str, kind: ManifestKind) {
    assert_eq!(
        classify_document(&DocumentInput {
            uri: uri.to_owned(),
            language_id: language_id.to_owned(),
            text: String::new(),
            workspace_root: None,
        }),
        kind
    );
}

#[test]
fn classifies_package_like_custom_json_as_npm() {
    for text in [
        r#"{"devDependencies":{"typescript":"^6.0.3"}}"#,
        r#"{"jspm":{"dependencies":{"systemjs":"^6.0.0"}}}"#,
        r#"{"workspaces":{"catalog":{"react":"^19.0.0"}}}"#,
    ] {
        assert_eq!(
            classify_document(&DocumentInput {
                uri: "file:///work/web-module.json".to_owned(),
                language_id: "json".to_owned(),
                text: text.to_owned(),
                workspace_root: None,
            }),
            ManifestKind::NpmPackageJson
        );
    }
}

#[test]
fn classifies_case_insensitive_manifest_extensions() {
    assert_eq!(
        classify_document(&DocumentInput {
            uri: "file:///work/PACKAGE.JSON".to_owned(),
            language_id: "json".to_owned(),
            text: String::new(),
            workspace_root: None,
        }),
        ManifestKind::NpmPackageJson
    );
    assert_eq!(
        classify_document(&DocumentInput {
            uri: "file:///work/DENO.JSONC".to_owned(),
            language_id: "jsonc".to_owned(),
            text: String::new(),
            workspace_root: None,
        }),
        ManifestKind::DenoJson
    );
    assert_eq!(
        classify_document(&DocumentInput {
            uri: "file:///work/app.CSPROJ".to_owned(),
            language_id: "xml".to_owned(),
            text: String::new(),
            workspace_root: None,
        }),
        ManifestKind::DotnetXml
    );
    assert_eq!(
        classify_document(&DocumentInput {
            uri: "file:///work/Requirements.TXT".to_owned(),
            language_id: "plaintext".to_owned(),
            text: String::new(),
            workspace_root: None,
        }),
        ManifestKind::PythonRequirementsTxt
    );
    assert_eq!(
        classify_document(&DocumentInput {
            uri: "file:///work/PIPFILE".to_owned(),
            language_id: "toml".to_owned(),
            text: String::new(),
            workspace_root: None,
        }),
        ManifestKind::PythonPipfile
    );
    assert_eq!(
        classify_document(&DocumentInput {
            uri: "file:///work/PYPROJECT.TOML".to_owned(),
            language_id: "toml".to_owned(),
            text: String::new(),
            workspace_root: None,
        }),
        ManifestKind::PythonPyprojectToml
    );
}

#[test]
fn classifies_case_insensitive_docker_and_workspace_manifests() {
    for (uri, language_id, kind) in [
        (
            "file:///work/COMPOSE.YAML",
            "yaml",
            ManifestKind::DockerComposeYaml,
        ),
        (
            "file:///work/DOCKER-COMPOSE.OVERRIDE.YML",
            "yaml",
            ManifestKind::DockerComposeYaml,
        ),
        (
            "file:///work/SERVICE.COMPOSE.YML",
            "yaml",
            ManifestKind::DockerComposeYaml,
        ),
        (
            "file:///work/DOCKERFILE",
            "dockerfile",
            ManifestKind::Dockerfile,
        ),
        (
            "file:///work/build.DOCKERFILE",
            "dockerfile",
            ManifestKind::Dockerfile,
        ),
        (
            "file:///work/PNPM-WORKSPACE.YAML",
            "yaml",
            ManifestKind::PnpmYaml,
        ),
        ("file:///work/.YARNRC.YML", "yaml", ManifestKind::PnpmYaml),
    ] {
        assert_manifest(uri, language_id, kind);
    }
}

#[test]
fn does_not_classify_generated_dotnet_outputs() {
    for uri in [
        "file:///work/obj/project.assets.props",
        "file:///work/bin/Debug/net8.0/app.targets",
        "file:///work/OBJ/Debug/net8.0/generated.props",
        "file:///work/BIN/Debug/net8.0/generated.targets",
    ] {
        assert_eq!(
            classify_document(&DocumentInput {
                uri: uri.to_owned(),
                language_id: "xml".to_owned(),
                text: String::new(),
                workspace_root: None,
            }),
            ManifestKind::Unknown,
        );
    }
}

#[test]
fn does_not_classify_manifest_name_suffixes() {
    for uri in [
        "file:///work/mycomposer.json",
        "file:///work/notpackage.json",
        "file:///work/appgo.mod",
        "file:///work/testpom.xml",
        "file:///work/otherpubspec.yaml",
    ] {
        assert_eq!(
            classify_document(&DocumentInput {
                uri: uri.to_owned(),
                language_id: "plaintext".to_owned(),
                text: String::new(),
                workspace_root: None,
            }),
            ManifestKind::Unknown,
        );
    }
}
