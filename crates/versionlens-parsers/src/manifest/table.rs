use crate::ansible_galaxy::parse_ansible_galaxy_requirements_yaml_with_paths;
use crate::bazel_module::parse_bazel_module_with_paths;
use crate::cargo_toml::parse_cargo_toml_with_paths;
use crate::clojure_deps::parse_clojure_deps_edn;
use crate::cocoapods_podfile::parse_cocoapods_podfile;
use crate::conan::{parse_conanfile_py, parse_conanfile_txt};
use crate::cpanfile::parse_cpanfile;
use crate::cpp::{parse_bazel_workspace, parse_cmake, parse_meson_wrap, parse_xmake_lua};
use crate::docker::{parse_docker_compose_yaml, parse_dockerfile};
use crate::dotnet_xml::parse_dotnet_xml_with_paths;
use crate::dub_sdl::parse_dub_sdl;
use crate::dune_project::parse_dune_project;
use crate::gemfile::{parse_gemfile, parse_gemspec};
use crate::github_actions::parse_github_actions;
use crate::gleam_toml::parse_gleam_toml;
use crate::go_mod::parse_go_mod;
use crate::gradle::{parse_gradle_build, parse_gradle_settings, parse_gradle_version_catalog_toml};
use crate::hackage::{parse_cabal, parse_cabal_project, parse_stack_yaml};
use crate::haxelib::parse_haxelib_json_with_paths;
use crate::helm_chart::parse_helm_chart_yaml_with_paths;
use crate::json_manifest::{
    parse_composer_json_with_paths, parse_deno_json_with_paths,
    parse_dotnet_project_json_with_paths, parse_dub_json_with_paths, parse_jsr_json_with_paths,
    parse_package_json_with_paths,
};
use crate::julia::{parse_julia_manifest, parse_julia_project};
use crate::kustomization_yaml::parse_kustomization_yaml_with_paths;
use crate::leiningen_project::parse_leiningen_project_clj;
use crate::luarocks::parse_luarocks_rockspec;
use crate::maven_xml::parse_maven_xml_with_paths;
use crate::mix_exs::parse_mix_exs;
use crate::nimble::parse_nimble;
use crate::nix_flake::parse_nix_flake_with_paths;
use crate::opam::parse_opam;
use crate::paket::{parse_paket_dependencies, parse_paket_references};
use crate::pnpm_yaml::parse_pnpm_yaml_with_paths;
use crate::pubspec_yaml::parse_pubspec_yaml_with_paths;
use crate::pyproject_toml::{parse_pipfile_with_paths, parse_pyproject_toml_with_paths};
use crate::r_description::{parse_r_description, parse_renv_lock};
use crate::rebar_config::parse_rebar_config;
use crate::requirements_txt::parse_requirements_txt;
use crate::runtime_manifests::{
    parse_bun_version, parse_node_version, parse_nvmrc, parse_rust_toolchain,
    parse_rust_toolchain_toml,
};
use crate::sbt_build::parse_sbt_build;
use crate::swift_package::parse_swift_package;
use crate::terraform_hcl::parse_terraform_hcl;
use crate::unity_manifest::parse_unity_project_manifest_json_with_paths;
use crate::vcpkg::parse_vcpkg_json_with_paths;
use crate::zig_zon::parse_zig_build_zon;
use versionlens_model::Ecosystem;
use versionlens_model::ManifestKind::{
    AnsibleGalaxyRequirementsYaml, BazelModule, BazelWorkspace, BunVersion, Cabal, CabalProject,
    CargoConfigToml, CargoToml, ClojureDepsEdn, Cmake, CocoaPodsPodfile, ComposerJson, ConanfilePy,
    ConanfileTxt, Cpanfile, DenoImportMapJson, DenoJson, DockerComposeYaml, Dockerfile,
    DotnetProjectJson, DotnetToolsJson, DotnetXml, DubJson, DubSdl, DuneProject, Gemfile,
    GitHubActions, GleamToml, GoMod, GradleBuild, GradleSettings, GradleVersionCatalogToml,
    HaxelibJson, HelmChartYaml, JsrJson, JuliaManifestToml, JuliaProjectToml, KustomizationYaml,
    LeiningenProjectClj, LuaRockspec, MavenPomXml, MesonWrap, MixExs, Nimble, NixFlake,
    NodeVersion, NpmPackageJson, NpmPackageJson5, NpmPackageYaml, Nvmrc, Opam, PaketDependencies,
    PaketReferences, PnpmYaml, PubspecOverridesYaml, PubspecYaml, PythonPipfile,
    PythonPyprojectToml, PythonRequirementsTxt, RDescription, RebarConfig, RenvLock, RubyGemspec,
    RustToolchain, RustToolchainToml, SbtBuild, StackYaml, SwiftPackage, TerraformTf,
    UnityProjectManifestJson, VcpkgJson, XmakeLua, ZigBuildZon,
};

use super::ManifestFormat::{Custom, Json, Toml, Xml, Yaml};
use super::ManifestParser::{Direct as ParserDirect, WithPaths as ParserWithPaths};
use super::Rule::{Content, Extension, Language, Name, Uri};
use super::Stage::{Content as ContentStage, Early, Editor, Late, Pubspec, Python, Workflow};
use super::default_paths;
use super::patterns::{
    is_cargo_config_uri, is_docker_compose_uri, is_dockerfile_uri, is_dotnet_xml_uri,
    is_github_actions_uri, is_julia_versioned_manifest_uri, is_package_json_content,
    is_requirements_txt_uri, is_unity_project_manifest_uri,
};
use super::{ManifestFormat, ManifestKind, ManifestParser, ManifestRow, Pattern, Rule, Stage};

const fn at(stage: Stage, rule: Rule) -> Pattern {
    Pattern { stage, rule }
}

const fn row(
    kind: ManifestKind,
    ecosystem: Ecosystem,
    provider: &'static str,
    format: ManifestFormat,
    patterns: &'static [Pattern],
    parser: ManifestParser,
) -> ManifestRow {
    ManifestRow {
        kind,
        ecosystem,
        provider,
        format,
        patterns,
        parser,
    }
}

/// Every supported file type. Within a stage, an earlier row wins.
pub static MANIFESTS: &[ManifestRow] = &[
    row(
        Nvmrc,
        Ecosystem::Npm,
        "npm",
        Custom,
        &[at(Early, Name(".nvmrc"))],
        ParserDirect(parse_nvmrc),
    ),
    row(
        NodeVersion,
        Ecosystem::Npm,
        "npm",
        Custom,
        &[at(Early, Name(".node-version"))],
        ParserDirect(parse_node_version),
    ),
    row(
        BunVersion,
        Ecosystem::Npm,
        "npm",
        Custom,
        &[at(Early, Name(".bun-version"))],
        ParserDirect(parse_bun_version),
    ),
    row(
        RustToolchain,
        Ecosystem::Cargo,
        "cargo",
        Custom,
        &[at(Early, Name("rust-toolchain"))],
        ParserDirect(parse_rust_toolchain),
    ),
    row(
        RustToolchainToml,
        Ecosystem::Cargo,
        "cargo",
        Toml,
        &[at(Early, Name("rust-toolchain.toml"))],
        ParserDirect(parse_rust_toolchain_toml),
    ),
    row(
        CargoToml,
        Ecosystem::Cargo,
        "cargo",
        Toml,
        &[at(Early, Name("Cargo.toml"))],
        ParserWithPaths(parse_cargo_toml_with_paths, default_paths::CARGO),
    ),
    row(
        CargoConfigToml,
        Ecosystem::Cargo,
        "cargo",
        Toml,
        &[at(Early, Uri(is_cargo_config_uri))],
        ParserWithPaths(parse_cargo_toml_with_paths, default_paths::CARGO_CONFIG),
    ),
    row(
        ComposerJson,
        Ecosystem::Composer,
        "composer",
        Json,
        &[at(Early, Name("composer.json"))],
        ParserWithPaths(parse_composer_json_with_paths, default_paths::COMPOSER),
    ),
    row(
        DenoJson,
        Ecosystem::Deno,
        "deno",
        Json,
        &[at(Early, Name("deno.json")), at(Early, Name("deno.jsonc"))],
        ParserWithPaths(parse_deno_json_with_paths, default_paths::DENO),
    ),
    row(
        DenoImportMapJson,
        Ecosystem::Deno,
        "deno",
        Json,
        &[at(Early, Name("import_map.json"))],
        ParserWithPaths(parse_deno_json_with_paths, default_paths::DENO),
    ),
    row(
        JsrJson,
        Ecosystem::Deno,
        "deno",
        Json,
        &[at(Early, Name("jsr.json")), at(Early, Name("jsr.jsonc"))],
        ParserWithPaths(parse_jsr_json_with_paths, default_paths::JSR),
    ),
    row(
        DotnetXml,
        Ecosystem::Dotnet,
        "dotnet",
        Xml,
        &[
            at(Early, Name("packages.config")),
            at(Early, Uri(is_dotnet_xml_uri)),
        ],
        ParserWithPaths(parse_dotnet_xml_with_paths, default_paths::DOTNET_XML),
    ),
    row(
        PaketDependencies,
        Ecosystem::Dotnet,
        "dotnet",
        Custom,
        &[at(Early, Name("paket.dependencies"))],
        ParserDirect(parse_paket_dependencies),
    ),
    row(
        PaketReferences,
        Ecosystem::Dotnet,
        "dotnet",
        Custom,
        &[at(Early, Name("paket.references"))],
        ParserDirect(parse_paket_references),
    ),
    row(
        DotnetProjectJson,
        Ecosystem::Dotnet,
        "dotnet",
        Json,
        &[at(Early, Name("project.json"))],
        ParserWithPaths(
            parse_dotnet_project_json_with_paths,
            default_paths::DOTNET_PROJECT_JSON,
        ),
    ),
    row(
        DotnetToolsJson,
        Ecosystem::Dotnet,
        "dotnet",
        Json,
        &[at(Early, Name("dotnet-tools.json"))],
        ParserWithPaths(
            parse_dotnet_project_json_with_paths,
            default_paths::DOTNET_TOOLS,
        ),
    ),
    row(
        GradleVersionCatalogToml,
        Ecosystem::Maven,
        "maven",
        Toml,
        &[at(Early, Name("libs.versions.toml"))],
        ParserDirect(parse_gradle_version_catalog_toml),
    ),
    row(
        GradleBuild,
        Ecosystem::Maven,
        "maven",
        Custom,
        &[
            at(Early, Name("build.gradle")),
            at(Early, Name("build.gradle.kts")),
        ],
        ParserDirect(parse_gradle_build),
    ),
    row(
        GradleSettings,
        Ecosystem::Maven,
        "maven",
        Custom,
        &[
            at(Early, Name("settings.gradle")),
            at(Early, Name("settings.gradle.kts")),
        ],
        ParserDirect(parse_gradle_settings),
    ),
    row(
        SbtBuild,
        Ecosystem::Maven,
        "maven",
        Custom,
        &[at(Early, Name("build.sbt"))],
        ParserDirect(parse_sbt_build),
    ),
    row(
        ClojureDepsEdn,
        Ecosystem::Maven,
        "maven",
        Custom,
        &[at(Early, Name("deps.edn"))],
        ParserDirect(parse_clojure_deps_edn),
    ),
    row(
        LeiningenProjectClj,
        Ecosystem::Maven,
        "maven",
        Custom,
        &[at(Early, Name("project.clj"))],
        ParserDirect(parse_leiningen_project_clj),
    ),
    row(
        MavenPomXml,
        Ecosystem::Maven,
        "maven",
        Xml,
        &[at(Early, Name("pom.xml"))],
        ParserWithPaths(parse_maven_xml_with_paths, default_paths::MAVEN),
    ),
    row(
        MixExs,
        Ecosystem::Hex,
        "hex",
        Custom,
        &[at(Early, Name("mix.exs"))],
        ParserDirect(parse_mix_exs),
    ),
    row(
        RebarConfig,
        Ecosystem::Hex,
        "hex",
        Custom,
        &[at(Early, Name("rebar.config"))],
        ParserDirect(parse_rebar_config),
    ),
    row(
        GleamToml,
        Ecosystem::Hex,
        "hex",
        Toml,
        &[at(Early, Name("gleam.toml"))],
        ParserDirect(parse_gleam_toml),
    ),
    row(
        Opam,
        Ecosystem::Opam,
        "opam",
        Custom,
        &[at(Early, Name("opam")), at(Early, Extension("opam"))],
        ParserDirect(parse_opam),
    ),
    row(
        DuneProject,
        Ecosystem::Opam,
        "opam",
        Custom,
        &[at(Early, Name("dune-project"))],
        ParserDirect(parse_dune_project),
    ),
    row(
        CabalProject,
        Ecosystem::Hackage,
        "hackage",
        Custom,
        &[at(Early, Name("cabal.project"))],
        ParserDirect(parse_cabal_project),
    ),
    row(
        Cabal,
        Ecosystem::Hackage,
        "hackage",
        Custom,
        &[at(Early, Extension("cabal"))],
        ParserDirect(parse_cabal),
    ),
    row(
        StackYaml,
        Ecosystem::Hackage,
        "hackage",
        Yaml,
        &[at(Early, Name("stack.yaml")), at(Early, Name("stack.yml"))],
        ParserDirect(parse_stack_yaml),
    ),
    row(
        JuliaProjectToml,
        Ecosystem::Julia,
        "julia",
        Toml,
        &[at(Early, Name("Project.toml"))],
        ParserDirect(parse_julia_project),
    ),
    row(
        JuliaManifestToml,
        Ecosystem::Julia,
        "julia",
        Toml,
        &[
            at(Early, Name("Manifest.toml")),
            at(Early, Uri(is_julia_versioned_manifest_uri)),
        ],
        ParserDirect(parse_julia_manifest),
    ),
    row(
        RDescription,
        Ecosystem::Cran,
        "cran",
        Custom,
        &[at(Early, Name("DESCRIPTION"))],
        ParserDirect(parse_r_description),
    ),
    row(
        RenvLock,
        Ecosystem::Cran,
        "cran",
        Json,
        &[at(Early, Name("renv.lock"))],
        ParserDirect(parse_renv_lock),
    ),
    row(
        ConanfileTxt,
        Ecosystem::Conan,
        "conan",
        Custom,
        &[at(Early, Name("conanfile.txt"))],
        ParserDirect(parse_conanfile_txt),
    ),
    row(
        ConanfilePy,
        Ecosystem::Conan,
        "conan",
        Custom,
        &[at(Early, Name("conanfile.py"))],
        ParserDirect(parse_conanfile_py),
    ),
    row(
        VcpkgJson,
        Ecosystem::Vcpkg,
        "vcpkg",
        Json,
        &[at(Early, Name("vcpkg.json"))],
        ParserWithPaths(parse_vcpkg_json_with_paths, default_paths::VCPKG),
    ),
    row(
        Cmake,
        Ecosystem::Cpp,
        "cpp",
        Custom,
        &[
            at(Early, Name("CMakeLists.txt")),
            at(Early, Extension("cmake")),
        ],
        ParserDirect(parse_cmake),
    ),
    row(
        XmakeLua,
        Ecosystem::Cpp,
        "cpp",
        Custom,
        &[at(Early, Name("xmake.lua"))],
        ParserDirect(parse_xmake_lua),
    ),
    row(
        MesonWrap,
        Ecosystem::Cpp,
        "cpp",
        Custom,
        &[at(Early, Extension("wrap"))],
        ParserDirect(parse_meson_wrap),
    ),
    row(
        BazelWorkspace,
        Ecosystem::Cpp,
        "cpp",
        Custom,
        &[
            at(Early, Name("WORKSPACE")),
            at(Early, Name("WORKSPACE.bazel")),
        ],
        ParserDirect(parse_bazel_workspace),
    ),
    row(
        SwiftPackage,
        Ecosystem::Swift,
        "swift",
        Custom,
        &[at(Early, Name("Package.swift"))],
        ParserDirect(parse_swift_package),
    ),
    row(
        ZigBuildZon,
        Ecosystem::Zig,
        "zig",
        Custom,
        &[at(Early, Name("build.zig.zon"))],
        ParserDirect(parse_zig_build_zon),
    ),
    row(
        Nimble,
        Ecosystem::Nim,
        "nim",
        Custom,
        &[at(Early, Extension("nimble"))],
        ParserDirect(parse_nimble),
    ),
    row(
        LuaRockspec,
        Ecosystem::LuaRocks,
        "luarocks",
        Custom,
        &[at(Early, Extension("rockspec"))],
        ParserDirect(parse_luarocks_rockspec),
    ),
    row(
        Cpanfile,
        Ecosystem::Cpan,
        "cpan",
        Custom,
        &[at(Early, Name("cpanfile"))],
        ParserDirect(parse_cpanfile),
    ),
    row(
        HaxelibJson,
        Ecosystem::Haxelib,
        "haxelib",
        Json,
        &[at(Early, Name("haxelib.json"))],
        ParserWithPaths(parse_haxelib_json_with_paths, default_paths::HAXELIB),
    ),
    row(
        TerraformTf,
        Ecosystem::Terraform,
        "terraform",
        Custom,
        &[at(Early, Extension("tf")), at(Early, Extension("tofu"))],
        ParserDirect(parse_terraform_hcl),
    ),
    row(
        HelmChartYaml,
        Ecosystem::Helm,
        "helm",
        Yaml,
        &[at(Early, Name("Chart.yaml"))],
        ParserWithPaths(parse_helm_chart_yaml_with_paths, default_paths::NONE),
    ),
    row(
        AnsibleGalaxyRequirementsYaml,
        Ecosystem::AnsibleGalaxy,
        "ansible",
        Yaml,
        &[
            at(Early, Name("requirements.yml")),
            at(Early, Name("requirements.yaml")),
        ],
        ParserWithPaths(
            parse_ansible_galaxy_requirements_yaml_with_paths,
            default_paths::NONE,
        ),
    ),
    row(
        BazelModule,
        Ecosystem::Bazel,
        "bazel",
        Custom,
        &[at(Early, Name("MODULE.bazel"))],
        ParserWithPaths(parse_bazel_module_with_paths, default_paths::NONE),
    ),
    row(
        NixFlake,
        Ecosystem::Nix,
        "nix",
        Custom,
        &[at(Early, Name("flake.nix"))],
        ParserWithPaths(parse_nix_flake_with_paths, default_paths::NONE),
    ),
    row(
        KustomizationYaml,
        Ecosystem::Docker,
        "kustomize",
        Yaml,
        &[
            at(Early, Name("kustomization.yaml")),
            at(Early, Name("kustomization.yml")),
            at(Early, Name("Kustomization")),
        ],
        ParserWithPaths(parse_kustomization_yaml_with_paths, default_paths::NONE),
    ),
    row(
        CocoaPodsPodfile,
        Ecosystem::CocoaPods,
        "cocoapods",
        Custom,
        &[at(Early, Name("Podfile"))],
        ParserDirect(parse_cocoapods_podfile),
    ),
    row(
        UnityProjectManifestJson,
        Ecosystem::Unity,
        "unity",
        Json,
        &[at(Early, Uri(is_unity_project_manifest_uri))],
        ParserWithPaths(
            parse_unity_project_manifest_json_with_paths,
            default_paths::NONE,
        ),
    ),
    row(
        RubyGemspec,
        Ecosystem::Ruby,
        "ruby",
        Custom,
        &[at(Early, Extension("gemspec"))],
        ParserDirect(parse_gemspec),
    ),
    row(
        PnpmYaml,
        Ecosystem::Npm,
        "pnpm",
        Yaml,
        &[
            at(Early, Name("pnpm-workspace.yaml")),
            at(Early, Name("pnpm-workspace.yml")),
            at(Early, Name(".yarnrc.yml")),
            at(Early, Name(".yarnrc.yaml")),
        ],
        ParserWithPaths(parse_pnpm_yaml_with_paths, default_paths::PNPM),
    ),
    row(
        DockerComposeYaml,
        Ecosystem::Docker,
        "docker",
        Yaml,
        &[
            at(Early, Uri(is_docker_compose_uri)),
            at(Editor, Language("dockercompose")),
        ],
        ParserDirect(parse_docker_compose_yaml),
    ),
    row(
        Dockerfile,
        Ecosystem::Docker,
        "docker",
        Custom,
        &[
            at(Editor, Language("dockerfile")),
            at(Editor, Uri(is_dockerfile_uri)),
        ],
        ParserDirect(parse_dockerfile),
    ),
    row(
        GitHubActions,
        Ecosystem::GitHub,
        "github",
        Yaml,
        &[at(Workflow, Uri(is_github_actions_uri))],
        ParserDirect(parse_github_actions),
    ),
    row(
        DubJson,
        Ecosystem::Dub,
        "dub",
        Json,
        &[
            at(Late, Name("dub.json")),
            at(Late, Name("dub.selections.json")),
        ],
        ParserWithPaths(parse_dub_json_with_paths, default_paths::DUB),
    ),
    row(
        DubSdl,
        Ecosystem::Dub,
        "dub",
        Custom,
        &[at(Late, Name("dub.sdl"))],
        ParserDirect(parse_dub_sdl),
    ),
    row(
        Gemfile,
        Ecosystem::Ruby,
        "ruby",
        Custom,
        &[at(Late, Name("Gemfile"))],
        ParserDirect(parse_gemfile),
    ),
    row(
        GoMod,
        Ecosystem::Go,
        "golang",
        Custom,
        &[at(Late, Name("go.mod")), at(Late, Name("go.work"))],
        ParserDirect(parse_go_mod),
    ),
    row(
        NpmPackageJson,
        Ecosystem::Npm,
        "npm",
        Json,
        &[
            at(Late, Name("package.json")),
            at(ContentStage, Content(is_package_json_content)),
        ],
        ParserWithPaths(parse_package_json_with_paths, default_paths::NPM),
    ),
    row(
        NpmPackageJson5,
        Ecosystem::Npm,
        "npm",
        Json,
        &[at(Late, Name("package.json5"))],
        ParserWithPaths(parse_package_json_with_paths, default_paths::NPM),
    ),
    row(
        NpmPackageYaml,
        Ecosystem::Npm,
        "npm",
        Yaml,
        &[
            at(Late, Name("package.yaml")),
            at(Late, Name("package.yml")),
        ],
        ParserWithPaths(parse_pnpm_yaml_with_paths, default_paths::PNPM),
    ),
    row(
        PythonRequirementsTxt,
        Ecosystem::Python,
        "pypi",
        Custom,
        &[
            at(Python, Language("pip-requirements")),
            at(Python, Uri(is_requirements_txt_uri)),
        ],
        ParserDirect(parse_requirements_txt),
    ),
    row(
        PythonPipfile,
        Ecosystem::Python,
        "pypi",
        Toml,
        &[at(Python, Name("Pipfile"))],
        ParserWithPaths(parse_pipfile_with_paths, default_paths::PYPI),
    ),
    row(
        PythonPyprojectToml,
        Ecosystem::Python,
        "pypi",
        Toml,
        &[at(Python, Name("pyproject.toml"))],
        ParserWithPaths(parse_pyproject_toml_with_paths, default_paths::PYPI),
    ),
    row(
        PubspecOverridesYaml,
        Ecosystem::Pub,
        "pub",
        Yaml,
        &[at(Pubspec, Name("pubspec_overrides.yaml"))],
        ParserWithPaths(
            parse_pubspec_yaml_with_paths,
            default_paths::PUBSPEC_OVERRIDES,
        ),
    ),
    row(
        PubspecYaml,
        Ecosystem::Pub,
        "pub",
        Yaml,
        &[
            at(Pubspec, Name("pubspec.yaml")),
            at(Pubspec, Name("pubspec.yml")),
        ],
        ParserWithPaths(parse_pubspec_yaml_with_paths, default_paths::PUBSPEC),
    ),
];
