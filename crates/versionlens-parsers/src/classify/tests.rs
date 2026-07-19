use std::fs::read_to_string;
use std::path::PathBuf;
use versionlens_model::{DocumentInput, ManifestKind};

use super::classify_document;
use versionlens_model::ManifestKind::{
    AnsibleGalaxyRequirementsYaml, BazelModule, BazelWorkspace, Cabal, CabalProject, CargoToml,
    ClojureDepsEdn, Cmake, ComposerJson, Cpanfile, DenoImportMapJson, DenoJson, DockerComposeYaml,
    Dockerfile, DotnetProjectJson, DotnetXml, DubJson, DubSdl, DuneProject, Gemfile, GleamToml,
    GoMod, GradleBuild, GradleSettings, GradleVersionCatalogToml, HaxelibJson, HelmChartYaml,
    JsrJson, JuliaManifestToml, JuliaProjectToml, KustomizationYaml, LeiningenProjectClj,
    LuaRockspec, MavenPomXml, MesonWrap, MixExs, Nimble, NixFlake, NpmPackageJson, NpmPackageJson5,
    NpmPackageYaml, Opam, PaketDependencies, PaketReferences, PnpmYaml, PubspecOverridesYaml,
    PubspecYaml, PythonPipfile, PythonPyprojectToml, PythonRequirementsTxt, RDescription,
    RebarConfig, RenvLock, RubyGemspec, SbtBuild, StackYaml, SwiftPackage, TerraformTf,
    UnityProjectManifestJson, Unknown, VcpkgJson, VersionLensMultiRegistries, XmakeLua,
    ZigBuildZon,
};

#[test]
fn classifies_supported_json_toml_and_xml_manifest_files() {
    for (uri, language_id, kind) in [
        ("file:///work/package.json", "jsonc", NpmPackageJson),
        ("file:///work/package.json5", "json5", NpmPackageJson5),
        ("file:///work/package.yaml", "yaml", NpmPackageYaml),
        ("file:///work/Cargo.toml", "toml", CargoToml),
        ("file:///work/composer.json", "json", ComposerJson),
        ("file:///work/deno.json", "jsonc", DenoJson),
        ("file:///work/deno.jsonc", "jsonc", DenoJson),
        ("file:///work/import_map.json", "json", DenoImportMapJson),
        ("file:///work/jsr.json", "json", JsrJson),
        ("file:///work/jsr.jsonc", "jsonc", JsrJson),
        ("file:///work/project.json", "json", DotnetProjectJson),
        ("file:///work/packages.config", "xml", DotnetXml),
        (
            "file:///work/paket.dependencies",
            "plaintext",
            PaketDependencies,
        ),
        (
            "file:///work/paket.references",
            "plaintext",
            PaketReferences,
        ),
        ("file:///work/app.csproj", "xml", DotnetXml),
        ("file:///work/app.fsproj", "xml", DotnetXml),
        ("file:///work/app.vbproj", "xml", DotnetXml),
        ("file:///work/Directory.Packages.props", "xml", DotnetXml),
        ("file:///work/Directory.Build.targets", "xml", DotnetXml),
        ("file:///work/dub.json", "json", DubJson),
        ("file:///work/vcpkg.json", "json", VcpkgJson),
        ("file:///work/CMakeLists.txt", "cmake", Cmake),
        ("file:///work/toolchain.cmake", "cmake", Cmake),
        ("file:///work/xmake.lua", "lua", XmakeLua),
        ("file:///work/Package.swift", "swift", SwiftPackage),
        ("file:///work/build.zig.zon", "zig", ZigBuildZon),
        ("file:///work/demo.nimble", "nim", Nimble),
        ("file:///work/cpanfile", "perl", Cpanfile),
        (
            "file:///work/luasocket-3.1.0-1.rockspec",
            "lua",
            LuaRockspec,
        ),
        ("file:///work/Pipfile", "toml", PythonPipfile),
        ("file:///work/pyproject.toml", "toml", PythonPyprojectToml),
        (
            "file:///work/gradle/libs.versions.toml",
            "toml",
            GradleVersionCatalogToml,
        ),
        ("file:///work/build.gradle", "groovy", GradleBuild),
        ("file:///work/build.gradle.kts", "kotlin", GradleBuild),
        ("file:///work/settings.gradle", "groovy", GradleSettings),
        ("file:///work/settings.gradle.kts", "kotlin", GradleSettings),
        ("file:///work/build.sbt", "scala", SbtBuild),
        ("file:///work/deps.edn", "clojure", ClojureDepsEdn),
        ("file:///work/project.clj", "clojure", LeiningenProjectClj),
        ("file:///work/pom.xml", "xml", MavenPomXml),
    ] {
        assert_manifest(uri, language_id, kind);
    }
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "table-driven manifest coverage stays readable as one scenario"
)]
fn classifies_supported_yaml_plaintext_and_other_manifest_files() {
    for (uri, language_id, kind) in [
        ("file:///work/Dockerfile", "dockerfile", Dockerfile),
        ("file:///work/build.Dockerfile", "dockerfile", Dockerfile),
        ("file:///work/compose.yaml", "yaml", DockerComposeYaml),
        (
            "file:///work/docker-compose.yaml",
            "yaml",
            DockerComposeYaml,
        ),
        (
            "file:///work/docker-compose.override.yml",
            "dockercompose",
            DockerComposeYaml,
        ),
        (
            "file:///work/docker-compose.prod.yaml",
            "yaml",
            DockerComposeYaml,
        ),
        ("file:///work/Chart.yaml", "yaml", HelmChartYaml),
        (
            "file:///work/requirements.yml",
            "yaml",
            AnsibleGalaxyRequirementsYaml,
        ),
        (
            "file:///work/requirements.yaml",
            "yaml",
            AnsibleGalaxyRequirementsYaml,
        ),
        ("file:///work/MODULE.bazel", "starlark", BazelModule),
        ("file:///work/WORKSPACE", "starlark", BazelWorkspace),
        ("file:///work/WORKSPACE.bazel", "starlark", BazelWorkspace),
        ("file:///work/subprojects/zlib.wrap", "meson", MesonWrap),
        ("file:///work/flake.nix", "nix", NixFlake),
        ("file:///work/kustomization.yaml", "yaml", KustomizationYaml),
        (
            "file:///work/Packages/manifest.json",
            "json",
            UnityProjectManifestJson,
        ),
        (
            "file:///work/compose.override.yaml",
            "yaml",
            DockerComposeYaml,
        ),
        ("file:///work/Gemfile", "ruby", Gemfile),
        ("file:///work/example.gemspec", "ruby", RubyGemspec),
        ("file:///work/go.mod", "go.mod", GoMod),
        ("file:///work/go.work", "go.mod", GoMod),
        ("file:///work/dub.sdl", "plaintext", DubSdl),
        ("file:///work/mix.exs", "elixir", MixExs),
        ("file:///work/rebar.config", "erlang", RebarConfig),
        ("file:///work/gleam.toml", "toml", GleamToml),
        ("file:///work/opam", "plaintext", Opam),
        ("file:///work/lwt.opam", "plaintext", Opam),
        ("file:///work/dune-project", "plaintext", DuneProject),
        ("file:///work/demo.cabal", "plaintext", Cabal),
        ("file:///work/cabal.project", "plaintext", CabalProject),
        ("file:///work/stack.yaml", "yaml", StackYaml),
        ("file:///work/Project.toml", "toml", JuliaProjectToml),
        ("file:///work/Manifest.toml", "toml", JuliaManifestToml),
        (
            "file:///work/Manifest-v1.11.toml",
            "toml",
            JuliaManifestToml,
        ),
        (
            "file:///work/Manifest-v1.10.toml",
            "toml",
            JuliaManifestToml,
        ),
        (
            "file:///work/requirements-dev.txt",
            "plaintext",
            PythonRequirementsTxt,
        ),
        (
            "file:///work/constraints.txt",
            "plaintext",
            PythonRequirementsTxt,
        ),
        ("file:///work/pubspec.yaml", "yaml", PubspecYaml),
        ("file:///work/pubspec.yml", "yaml", PubspecYaml),
        (
            "file:///work/pubspec_overrides.yaml",
            "yaml",
            PubspecOverridesYaml,
        ),
        ("file:///work/pnpm-workspace.yaml", "yaml", PnpmYaml),
        ("file:///work/pnpm-workspace.yml", "yaml", PnpmYaml),
        ("file:///work/.yarnrc.yaml", "yaml", PnpmYaml),
        ("file:///work/.yarnrc.yml", "yaml", PnpmYaml),
        (
            "file:///work/service.compose.yml",
            "yaml",
            DockerComposeYaml,
        ),
        ("file:///work/build.dockerfile", "dockerfile", Dockerfile),
        (
            "versionlens:/versionlens.multi-registries.json",
            "json",
            VersionLensMultiRegistries,
        ),
    ] {
        assert_manifest(uri, language_id, kind);
    }
}

#[test]
fn classifies_known_manifest_paths_without_language_ids() {
    let cases = [
        ("file:///work/Cargo.toml", CargoToml),
        ("file:///work/composer.json", ComposerJson),
        ("file:///work/deno.jsonc", DenoJson),
        ("file:///work/import_map.json", DenoImportMapJson),
        ("file:///work/jsr.json", JsrJson),
        ("file:///work/jsr.jsonc", JsrJson),
        ("file:///work/packages.config", DotnetXml),
        ("file:///work/paket.dependencies", PaketDependencies),
        ("file:///work/paket.references", PaketReferences),
        ("file:///work/project.csproj", DotnetXml),
        ("file:///work/project.vbproj", DotnetXml),
        ("file:///work/pom.xml", MavenPomXml),
        ("file:///work/docker-compose.yaml", DockerComposeYaml),
        (
            "file:///work/docker-compose.override.yml",
            DockerComposeYaml,
        ),
        ("file:///work/compose.override.yaml", DockerComposeYaml),
        ("file:///work/pnpm-workspace.yaml", PnpmYaml),
        ("file:///work/dub.selections.json", DubJson),
        ("file:///work/dub.sdl", DubSdl),
        ("file:///work/Gemfile", Gemfile),
        ("file:///work/example.gemspec", RubyGemspec),
        ("file:///work/go.mod", GoMod),
        ("file:///work/go.work", GoMod),
        ("file:///work/opam", Opam),
        ("file:///work/lwt.opam", Opam),
        ("file:///work/dune-project", DuneProject),
        ("file:///work/demo.cabal", Cabal),
        ("file:///work/cabal.project", CabalProject),
        ("file:///work/stack.yaml", StackYaml),
        ("file:///work/Project.toml", JuliaProjectToml),
        ("file:///work/Manifest.toml", JuliaManifestToml),
        ("file:///work/DESCRIPTION", RDescription),
        ("file:///work/renv.lock", RenvLock),
        ("file:///work/Manifest-v1.11.toml", JuliaManifestToml),
        ("file:///work/Manifest-v1.10.toml", JuliaManifestToml),
        ("file:///work/package.json", NpmPackageJson),
        ("file:///work/package.json5", NpmPackageJson5),
        ("file:///work/package.yaml", NpmPackageYaml),
        ("file:///work/CMakeLists.txt", Cmake),
        ("file:///work/toolchain.cmake", Cmake),
        ("file:///work/xmake.lua", XmakeLua),
        ("file:///work/subprojects/zlib.wrap", MesonWrap),
        ("file:///work/WORKSPACE", BazelWorkspace),
        ("file:///work/Pipfile", PythonPipfile),
        ("file:///work/pyproject.toml", PythonPyprojectToml),
        ("file:///work/pubspec.yaml", PubspecYaml),
        ("file:///work/pubspec_overrides.yaml", PubspecOverridesYaml),
    ];

    for (uri, expected) in cases {
        assert_eq!(
            classify_document(&DocumentInput {
                uri: uri.to_owned(),
                language_id: "plaintext".to_owned(),
                text: package_file_fixture(
                    "classifies-known-manifest-paths-without-language-ids.txt"
                )
                .to_owned(),
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
                text: package_file_fixture("ignores-ordinary-manifests-from-non-file-uris.txt")
                    .to_owned(),
                workspace_root: None,
            }),
            Unknown,
        );
    }

    assert_eq!(
        classify_document(&DocumentInput {
            uri: "versionlens:/versionlens.multi-registries.json".to_owned(),
            language_id: "json".to_owned(),
            text: String::new(),
            workspace_root: None,
        }),
        VersionLensMultiRegistries,
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
        package_file_fixture("package-like-dev-dependencies.json"),
        package_file_fixture("package-like-jspm-dependencies.json"),
        package_file_fixture("package-like-workspace-catalog.json"),
    ] {
        assert_eq!(
            classify_document(&DocumentInput {
                uri: "file:///work/web-module.json".to_owned(),
                language_id: "json".to_owned(),
                text: text.to_owned(),
                workspace_root: None,
            }),
            NpmPackageJson
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
        NpmPackageJson
    );
    assert_eq!(
        classify_document(&DocumentInput {
            uri: "file:///work/DENO.JSONC".to_owned(),
            language_id: "jsonc".to_owned(),
            text: String::new(),
            workspace_root: None,
        }),
        DenoJson
    );
    assert_eq!(
        classify_document(&DocumentInput {
            uri: "file:///work/app.CSPROJ".to_owned(),
            language_id: "xml".to_owned(),
            text: String::new(),
            workspace_root: None,
        }),
        DotnetXml
    );
    assert_eq!(
        classify_document(&DocumentInput {
            uri: "file:///work/Requirements.TXT".to_owned(),
            language_id: "plaintext".to_owned(),
            text: String::new(),
            workspace_root: None,
        }),
        PythonRequirementsTxt
    );
    assert_eq!(
        classify_document(&DocumentInput {
            uri: "file:///work/PIPFILE".to_owned(),
            language_id: "toml".to_owned(),
            text: String::new(),
            workspace_root: None,
        }),
        PythonPipfile
    );
    assert_eq!(
        classify_document(&DocumentInput {
            uri: "file:///work/PYPROJECT.TOML".to_owned(),
            language_id: "toml".to_owned(),
            text: String::new(),
            workspace_root: None,
        }),
        PythonPyprojectToml
    );
    assert_eq!(
        classify_document(&DocumentInput {
            uri: "file:///work/HAXELIB.JSON".to_owned(),
            language_id: "json".to_owned(),
            text: String::new(),
            workspace_root: None,
        }),
        HaxelibJson
    );
}

#[test]
fn classifies_case_insensitive_docker_and_workspace_manifests() {
    for (uri, language_id, kind) in [
        ("file:///work/COMPOSE.YAML", "yaml", DockerComposeYaml),
        (
            "file:///work/DOCKER-COMPOSE.OVERRIDE.YML",
            "yaml",
            DockerComposeYaml,
        ),
        (
            "file:///work/SERVICE.COMPOSE.YML",
            "yaml",
            DockerComposeYaml,
        ),
        ("file:///work/DOCKERFILE", "dockerfile", Dockerfile),
        ("file:///work/build.DOCKERFILE", "dockerfile", Dockerfile),
        ("file:///work/PNPM-WORKSPACE.YAML", "yaml", PnpmYaml),
        ("file:///work/.YARNRC.YML", "yaml", PnpmYaml),
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
            Unknown,
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
            Unknown,
        );
    }
}

#[test]
fn classifies_terraform_and_opentofu_files() {
    for uri in ["file:///work/main.tf", "file:///work/providers.tofu"] {
        assert_manifest(uri, "terraform", TerraformTf);
    }
}

fn package_file_fixture(name: &str) -> &'static str {
    let path = repo_root()
        .join("tests/fixtures/versionlens-parsers/src/classify/tests")
        .join(name);
    let contents = read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "failed to read package-file fixture {}: {error}",
            path.display()
        )
    });
    crate::leaked_string(contents)
}

fn repo_root() -> PathBuf {
    <PathBuf as From<&str>>::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("crate should be under crates/")
        .to_path_buf()
}
