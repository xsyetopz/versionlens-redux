use versionlens_model::ManifestKind::{Unknown, VersionLensMultiRegistries};
use versionlens_model::{DocumentInput, ManifestKind};

use crate::manifest::{MANIFESTS, Stage};

mod uri;

use uri::{SCHEMA_URI, document_uri, is_file_uri};

/// Picks the manifest kind of a document from its URI, language ID and text.
pub fn classify_document(input: &DocumentInput) -> ManifestKind {
    let uri = document_uri(&input.uri);
    if uri == SCHEMA_URI {
        return VersionLensMultiRegistries;
    }
    if !is_file_uri(uri) {
        return Unknown;
    }

    Stage::ALL
        .iter()
        .find_map(|&stage| {
            MANIFESTS.iter().find_map(|row| {
                row.patterns
                    .iter()
                    .any(|pattern| pattern.stage == stage && pattern.rule.matches(input, uri))
                    .then_some(row.kind)
            })
        })
        .unwrap_or(Unknown)
}

#[cfg(test)]
mod tests {
    use versionlens_model::DocumentInput;

    use super::classify_document;
    use versionlens_model::ManifestKind;

    mod cases {
        use versionlens_model::ManifestKind;

        pub(super) const JSON_TOML_XML_CASES: &[(&str, &str, ManifestKind)] = &[
            (
                "file:///work/rust-toolchain.toml",
                "toml",
                ManifestKind::RustToolchainToml,
            ),
            (
                "file:///work/package.json",
                "jsonc",
                ManifestKind::NpmPackageJson,
            ),
            (
                "file:///work/package.json5",
                "json5",
                ManifestKind::NpmPackageJson5,
            ),
            (
                "file:///work/package.yaml",
                "yaml",
                ManifestKind::NpmPackageYaml,
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
                "file:///work/import_map.json",
                "json",
                ManifestKind::DenoImportMapJson,
            ),
            ("file:///work/jsr.json", "json", ManifestKind::JsrJson),
            ("file:///work/jsr.jsonc", "jsonc", ManifestKind::JsrJson),
            (
                "file:///work/project.json",
                "json",
                ManifestKind::DotnetProjectJson,
            ),
            (
                "file:///work/packages.config",
                "xml",
                ManifestKind::DotnetXml,
            ),
            (
                "file:///work/paket.dependencies",
                "plaintext",
                ManifestKind::PaketDependencies,
            ),
            (
                "file:///work/paket.references",
                "plaintext",
                ManifestKind::PaketReferences,
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
            ("file:///work/vcpkg.json", "json", ManifestKind::VcpkgJson),
            ("file:///work/CMakeLists.txt", "cmake", ManifestKind::Cmake),
            ("file:///work/toolchain.cmake", "cmake", ManifestKind::Cmake),
            ("file:///work/xmake.lua", "lua", ManifestKind::XmakeLua),
            (
                "file:///work/Package.swift",
                "swift",
                ManifestKind::SwiftPackage,
            ),
            (
                "file:///work/build.zig.zon",
                "zig",
                ManifestKind::ZigBuildZon,
            ),
            ("file:///work/demo.nimble", "nim", ManifestKind::Nimble),
            ("file:///work/cpanfile", "perl", ManifestKind::Cpanfile),
            (
                "file:///work/luasocket-3.1.0-1.rockspec",
                "lua",
                ManifestKind::LuaRockspec,
            ),
            ("file:///work/Pipfile", "toml", ManifestKind::PythonPipfile),
            (
                "file:///work/pyproject.toml",
                "toml",
                ManifestKind::PythonPyprojectToml,
            ),
            (
                "file:///work/gradle/libs.versions.toml",
                "toml",
                ManifestKind::GradleVersionCatalogToml,
            ),
            (
                "file:///work/build.gradle",
                "groovy",
                ManifestKind::GradleBuild,
            ),
            (
                "file:///work/build.gradle.kts",
                "kotlin",
                ManifestKind::GradleBuild,
            ),
            (
                "file:///work/settings.gradle",
                "groovy",
                ManifestKind::GradleSettings,
            ),
            (
                "file:///work/settings.gradle.kts",
                "kotlin",
                ManifestKind::GradleSettings,
            ),
            ("file:///work/build.sbt", "scala", ManifestKind::SbtBuild),
            (
                "file:///work/deps.edn",
                "clojure",
                ManifestKind::ClojureDepsEdn,
            ),
            (
                "file:///work/project.clj",
                "clojure",
                ManifestKind::LeiningenProjectClj,
            ),
            ("file:///work/pom.xml", "xml", ManifestKind::MavenPomXml),
        ];
        pub(super) const YAML_PLAINTEXT_OTHER_CASES: &[(&str, &str, ManifestKind)] = &[
            ("file:///work/.nvmrc", "plaintext", ManifestKind::Nvmrc),
            (
                "file:///work/.node-version",
                "plaintext",
                ManifestKind::NodeVersion,
            ),
            (
                "file:///work/.bun-version",
                "plaintext",
                ManifestKind::BunVersion,
            ),
            (
                "file:///work/rust-toolchain",
                "plaintext",
                ManifestKind::RustToolchain,
            ),
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
                "file:///work/Chart.yaml",
                "yaml",
                ManifestKind::HelmChartYaml,
            ),
            (
                "file:///work/requirements.yml",
                "yaml",
                ManifestKind::AnsibleGalaxyRequirementsYaml,
            ),
            (
                "file:///work/requirements.yaml",
                "yaml",
                ManifestKind::AnsibleGalaxyRequirementsYaml,
            ),
            (
                "file:///work/MODULE.bazel",
                "starlark",
                ManifestKind::BazelModule,
            ),
            (
                "file:///work/WORKSPACE",
                "starlark",
                ManifestKind::BazelWorkspace,
            ),
            (
                "file:///work/WORKSPACE.bazel",
                "starlark",
                ManifestKind::BazelWorkspace,
            ),
            (
                "file:///work/subprojects/zlib.wrap",
                "meson",
                ManifestKind::MesonWrap,
            ),
            ("file:///work/flake.nix", "nix", ManifestKind::NixFlake),
            (
                "file:///work/kustomization.yaml",
                "yaml",
                ManifestKind::KustomizationYaml,
            ),
            (
                "file:///work/Packages/manifest.json",
                "json",
                ManifestKind::UnityProjectManifestJson,
            ),
            (
                "file:///work/compose.override.yaml",
                "yaml",
                ManifestKind::DockerComposeYaml,
            ),
            ("file:///work/Gemfile", "ruby", ManifestKind::Gemfile),
            (
                "file:///work/example.gemspec",
                "ruby",
                ManifestKind::RubyGemspec,
            ),
            ("file:///work/go.mod", "go.mod", ManifestKind::GoMod),
            ("file:///work/go.work", "go.mod", ManifestKind::GoMod),
            ("file:///work/dub.sdl", "plaintext", ManifestKind::DubSdl),
            ("file:///work/mix.exs", "elixir", ManifestKind::MixExs),
            (
                "file:///work/rebar.config",
                "erlang",
                ManifestKind::RebarConfig,
            ),
            ("file:///work/gleam.toml", "toml", ManifestKind::GleamToml),
            ("file:///work/opam", "plaintext", ManifestKind::Opam),
            ("file:///work/lwt.opam", "plaintext", ManifestKind::Opam),
            (
                "file:///work/dune-project",
                "plaintext",
                ManifestKind::DuneProject,
            ),
            ("file:///work/demo.cabal", "plaintext", ManifestKind::Cabal),
            (
                "file:///work/cabal.project",
                "plaintext",
                ManifestKind::CabalProject,
            ),
            ("file:///work/stack.yaml", "yaml", ManifestKind::StackYaml),
            (
                "file:///work/Project.toml",
                "toml",
                ManifestKind::JuliaProjectToml,
            ),
            (
                "file:///work/Manifest.toml",
                "toml",
                ManifestKind::JuliaManifestToml,
            ),
            (
                "file:///work/Manifest-v1.11.toml",
                "toml",
                ManifestKind::JuliaManifestToml,
            ),
            (
                "file:///work/Manifest-v1.10.toml",
                "toml",
                ManifestKind::JuliaManifestToml,
            ),
            (
                "file:///work/requirements-dev.txt",
                "plaintext",
                ManifestKind::PythonRequirementsTxt,
            ),
            (
                "file:///work/constraints.txt",
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
                "file:///work/pubspec_overrides.yaml",
                "yaml",
                ManifestKind::PubspecOverridesYaml,
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
                "versionlens:/multi-registries.json",
                "json",
                ManifestKind::VersionLensMultiRegistries,
            ),
        ];
    }
    mod fixtures {
        use std::fs;
        use std::path::PathBuf;

        use serde_json::Value;
        use versionlens_model::DocumentInput;

        use super::classify_document;
        use crate::{ecosystem_config_namespace, ecosystem_for_manifest};

        fn repository_path(relative: &str) -> PathBuf {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .join(relative)
        }

        fn read_json(relative: &str) -> Value {
            let path = repository_path(relative);
            let text = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
            serde_json::from_str(&text)
                .unwrap_or_else(|error| panic!("cannot parse {}: {error}", path.display()))
        }

        fn field<'a>(value: &'a Value, name: &str) -> &'a str {
            value[name]
                .as_str()
                .unwrap_or_else(|| panic!("missing string field {name} in {value}"))
        }

        /// Every manifest fixture classifies to the kind and ecosystem that its
        /// golden output records. The golden output names ecosystems by their
        /// settings namespace.
        #[test]
        fn classifies_every_fixture_to_its_golden_kind_and_ecosystem() {
            let cases = read_json("tests/fixtures/checking-coverage/manifest-cases.json");
            let cases = cases.as_array().expect("manifest cases are a list");
            assert_ne!(cases.len(), 0);

            for case in cases {
                let path = field(case, "path");
                let release = case["since"].as_str().unwrap_or("0.4.4");
                let golden = read_json(&format!("tests/golden/{release}/{path}.json"));
                let input = DocumentInput::new(
                    format!("file:///workspace/{path}"),
                    field(case, "language"),
                    field(case, "text"),
                    Some("file:///workspace".to_owned()),
                );

                let kind = classify_document(&input);
                assert_eq!(
                    format!("{kind:?}"),
                    field(&golden, "kind"),
                    "kind of {path}"
                );

                let ecosystem = ecosystem_for_manifest(kind).unwrap_or_else(|| {
                    panic!("{path} classified to {kind:?}, which has no ecosystem")
                });
                let golden_ecosystems = golden["dependencies"]
                    .as_array()
                    .expect("golden dependencies are a list")
                    .iter()
                    .map(|dependency| dependency["ecosystem"].clone())
                    .collect::<Vec<_>>();
                assert!(
                    !golden_ecosystems.is_empty(),
                    "{path} has no golden dependency"
                );
                for golden_ecosystem in golden_ecosystems {
                    assert_eq!(
                        Some(ecosystem_config_namespace(ecosystem)),
                        golden_ecosystem.as_str(),
                        "ecosystem of {path}"
                    );
                }
            }
        }
    }

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
                    package_file_fixture(
                        "classifies-known-manifest-paths-without-language-ids.txt"
                    )
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
}
