use versionlens_model::DocumentInput;

use super::classify_document;
use versionlens_model::ManifestKind;

mod cases;

use cases::{JSON_TOML_XML_CASES, YAML_PLAINTEXT_OTHER_CASES};

#[test]
fn classifies_supported_json_toml_and_xml_manifest_files() {
    for &(uri, language_id, kind) in JSON_TOML_XML_CASES {
        assert_manifest(uri, language_id, kind);
    }
}
#[test]
fn classifies_github_actions_only_in_supported_repository_locations() {
    assert_manifest(
        "file:///work/.github/workflows/ci.yml",
        "yaml",
        ManifestKind::GitHubActions,
    );
    assert_manifest(
        "file:///work/.github/actions/release/action.yaml",
        "yaml",
        ManifestKind::GitHubActions,
    );
    assert_manifest("file:///work/ci.yml", "yaml", ManifestKind::Unknown);
    assert_manifest(
        "file:///work/.github/workflows/README.md",
        "markdown",
        ManifestKind::Unknown,
    );
}

#[test]
fn classifies_supported_yaml_plaintext_and_other_manifest_files() {
    for &(uri, language_id, kind) in YAML_PLAINTEXT_OTHER_CASES {
        assert_manifest(uri, language_id, kind);
    }
}
#[test]
fn classifies_known_manifest_paths_without_language_ids() {
    let cases = [
        ("file:///work/Cargo.toml", ManifestKind::CargoToml),
        ("file:///work/composer.json", ManifestKind::ComposerJson),
        ("file:///work/deno.jsonc", ManifestKind::DenoJson),
        (
            "file:///work/import_map.json",
            ManifestKind::DenoImportMapJson,
        ),
        ("file:///work/jsr.json", ManifestKind::JsrJson),
        ("file:///work/jsr.jsonc", ManifestKind::JsrJson),
        ("file:///work/packages.config", ManifestKind::DotnetXml),
        (
            "file:///work/paket.dependencies",
            ManifestKind::PaketDependencies,
        ),
        (
            "file:///work/paket.references",
            ManifestKind::PaketReferences,
        ),
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
        ("file:///work/dub.sdl", ManifestKind::DubSdl),
        ("file:///work/Gemfile", ManifestKind::Gemfile),
        ("file:///work/example.gemspec", ManifestKind::RubyGemspec),
        ("file:///work/go.mod", ManifestKind::GoMod),
        ("file:///work/go.work", ManifestKind::GoMod),
        ("file:///work/opam", ManifestKind::Opam),
        ("file:///work/lwt.opam", ManifestKind::Opam),
        ("file:///work/dune-project", ManifestKind::DuneProject),
        ("file:///work/demo.cabal", ManifestKind::Cabal),
        ("file:///work/cabal.project", ManifestKind::CabalProject),
        ("file:///work/stack.yaml", ManifestKind::StackYaml),
        ("file:///work/Project.toml", ManifestKind::JuliaProjectToml),
        (
            "file:///work/Manifest.toml",
            ManifestKind::JuliaManifestToml,
        ),
        ("file:///work/DESCRIPTION", ManifestKind::RDescription),
        ("file:///work/renv.lock", ManifestKind::RenvLock),
        (
            "file:///work/Manifest-v1.11.toml",
            ManifestKind::JuliaManifestToml,
        ),
        (
            "file:///work/Manifest-v1.10.toml",
            ManifestKind::JuliaManifestToml,
        ),
        ("file:///work/package.json", ManifestKind::NpmPackageJson),
        ("file:///work/package.json5", ManifestKind::NpmPackageJson5),
        ("file:///work/package.yaml", ManifestKind::NpmPackageYaml),
        ("file:///work/CMakeLists.txt", ManifestKind::Cmake),
        ("file:///work/toolchain.cmake", ManifestKind::Cmake),
        ("file:///work/xmake.lua", ManifestKind::XmakeLua),
        (
            "file:///work/subprojects/zlib.wrap",
            ManifestKind::MesonWrap,
        ),
        ("file:///work/WORKSPACE", ManifestKind::BazelWorkspace),
        ("file:///work/Pipfile", ManifestKind::PythonPipfile),
        (
            "file:///work/pyproject.toml",
            ManifestKind::PythonPyprojectToml,
        ),
        ("file:///work/pubspec.yaml", ManifestKind::PubspecYaml),
        (
            "file:///work/pubspec_overrides.yaml",
            ManifestKind::PubspecOverridesYaml,
        ),
    ];

    for (uri, expected) in cases {
        assert_eq!(
            classify_document(&DocumentInput::new(
                uri.to_owned(),
                "plaintext".to_owned(),
                package_file_fixture("classifies-known-manifest-paths-without-language-ids.txt")
                    .to_owned(),
                None
            )),
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
            classify_document(&DocumentInput::new(
                uri.to_owned(),
                "json".to_owned(),
                package_file_fixture("ignores-ordinary-manifests-from-non-file-uris.txt")
                    .to_owned(),
                None
            )),
            ManifestKind::Unknown,
        );
    }

    assert_eq!(
        classify_document(&DocumentInput::new(
            "versionlens:/multi-registries.json".to_owned(),
            "json".to_owned(),
            String::new(),
            None
        )),
        ManifestKind::VersionLensMultiRegistries,
    );
}

fn assert_manifest(uri: &str, language_id: &str, kind: ManifestKind) {
    assert_eq!(
        classify_document(&DocumentInput::new(
            uri.to_owned(),
            language_id.to_owned(),
            String::new(),
            None
        )),
        kind
    );
}

#[test]
fn classifies_package_like_custom_json_as_npm() {
    for text in [
        package_file_fixture("like-dev-dependencies.json"),
        package_file_fixture("like-jspm-dependencies.json"),
        package_file_fixture("like-workspace-catalog.json"),
    ] {
        assert_eq!(
            classify_document(&DocumentInput::new(
                "file:///work/web-module.json".to_owned(),
                "json".to_owned(),
                text.to_owned(),
                None
            )),
            ManifestKind::NpmPackageJson
        );
    }
}

#[test]
fn classifies_case_insensitive_manifest_extensions() {
    assert_eq!(
        classify_document(&DocumentInput::new(
            "file:///work/PACKAGE.JSON".to_owned(),
            "json".to_owned(),
            String::new(),
            None
        )),
        ManifestKind::NpmPackageJson
    );
    assert_eq!(
        classify_document(&DocumentInput::new(
            "file:///work/DENO.JSONC".to_owned(),
            "jsonc".to_owned(),
            String::new(),
            None
        )),
        ManifestKind::DenoJson
    );
    assert_eq!(
        classify_document(&DocumentInput::new(
            "file:///work/app.CSPROJ".to_owned(),
            "xml".to_owned(),
            String::new(),
            None
        )),
        ManifestKind::DotnetXml
    );
    assert_eq!(
        classify_document(&DocumentInput::new(
            "file:///work/Requirements.TXT".to_owned(),
            "plaintext".to_owned(),
            String::new(),
            None
        )),
        ManifestKind::PythonRequirementsTxt
    );
    assert_eq!(
        classify_document(&DocumentInput::new(
            "file:///work/PIPFILE".to_owned(),
            "toml".to_owned(),
            String::new(),
            None
        )),
        ManifestKind::PythonPipfile
    );
    assert_eq!(
        classify_document(&DocumentInput::new(
            "file:///work/PYPROJECT.TOML".to_owned(),
            "toml".to_owned(),
            String::new(),
            None
        )),
        ManifestKind::PythonPyprojectToml
    );
    assert_eq!(
        classify_document(&DocumentInput::new(
            "file:///work/HAXELIB.JSON".to_owned(),
            "json".to_owned(),
            String::new(),
            None
        )),
        ManifestKind::HaxelibJson
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
            classify_document(&DocumentInput::new(
                uri.to_owned(),
                "xml".to_owned(),
                String::new(),
                None
            )),
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
            classify_document(&DocumentInput::new(
                uri.to_owned(),
                "plaintext".to_owned(),
                String::new(),
                None
            )),
            ManifestKind::Unknown,
        );
    }
}

#[test]
fn classifies_terraform_and_opentofu_files() {
    for uri in ["file:///work/main.tf", "file:///work/providers.tofu"] {
        assert_manifest(uri, "terraform", ManifestKind::TerraformTf);
    }
}

fn package_file_fixture(name: &str) -> &'static str {
    crate::support::tests::fixture(
        "tests/fixtures/versionlens-parsers/src/classify/tests",
        name,
    )
}
