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
use crate::sbt_build::parse_sbt_build;
use crate::swift_package::parse_swift_package;
use crate::terraform_hcl::parse_terraform_hcl;
use crate::unity_manifest::parse_unity_project_manifest_json_with_paths;
use crate::vcpkg::parse_vcpkg_json_with_paths;
use crate::zig_zon::parse_zig_build_zon;
use versionlens_model::ManifestKind::{
    AnsibleGalaxyRequirementsYaml, BazelModule, BazelWorkspace, Cabal, CabalProject, CargoToml,
    ClojureDepsEdn, Cmake, CocoaPodsPodfile, ComposerJson, ConanfilePy, ConanfileTxt, Cpanfile,
    DenoImportMapJson, DenoJson, DockerComposeYaml, Dockerfile, DotnetProjectJson, DotnetXml,
    DubJson, DubSdl, DuneProject, Gemfile, GleamToml, GoMod, GradleBuild, GradleSettings,
    GradleVersionCatalogToml, HaxelibJson, HelmChartYaml, JsrJson, JuliaManifestToml,
    JuliaProjectToml, KustomizationYaml, LeiningenProjectClj, LuaRockspec, MavenPomXml, MesonWrap,
    MixExs, Nimble, NixFlake, NpmPackageJson, NpmPackageJson5, NpmPackageYaml, Opam,
    PaketDependencies, PaketReferences, PnpmYaml, PubspecOverridesYaml, PubspecYaml, PythonPipfile,
    PythonPyprojectToml, PythonRequirementsTxt, RDescription, RebarConfig, RenvLock, RubyGemspec,
    SbtBuild, StackYaml, SwiftPackage, TerraformTf, UnityProjectManifestJson, VcpkgJson, XmakeLua,
    ZigBuildZon,
};
use versionlens_model::{Dependency, ManifestKind};

use self::ManifestParser::{Direct as ParserDirect, WithPaths as ParserWithPaths};

type ParsedDependencies = Vec<Dependency>;

#[derive(Clone, Copy)]
enum ManifestParser {
    Direct(fn(&str) -> ParsedDependencies),
    WithPaths(fn(&str, &[&str]) -> ParsedDependencies),
}

impl ManifestParser {
    fn parse(self, text: &str, paths: &[&str]) -> ParsedDependencies {
        match self {
            Self::Direct(parser) => parser(text),
            Self::WithPaths(parser) => parser(text, paths),
        }
    }
}

const MANIFEST_PARSERS: &[(ManifestKind, ManifestParser)] = &[
    (CargoToml, ParserWithPaths(parse_cargo_toml_with_paths)),
    (
        ComposerJson,
        ParserWithPaths(parse_composer_json_with_paths),
    ),
    (DenoJson, ParserWithPaths(parse_deno_json_with_paths)),
    (
        DenoImportMapJson,
        ParserWithPaths(parse_deno_json_with_paths),
    ),
    (JsrJson, ParserWithPaths(parse_jsr_json_with_paths)),
    (
        DotnetProjectJson,
        ParserWithPaths(parse_dotnet_project_json_with_paths),
    ),
    (DotnetXml, ParserWithPaths(parse_dotnet_xml_with_paths)),
    (PaketDependencies, ParserDirect(parse_paket_dependencies)),
    (PaketReferences, ParserDirect(parse_paket_references)),
    (DockerComposeYaml, ParserDirect(parse_docker_compose_yaml)),
    (Dockerfile, ParserDirect(parse_dockerfile)),
    (
        KustomizationYaml,
        ParserWithPaths(parse_kustomization_yaml_with_paths),
    ),
    (DubJson, ParserWithPaths(parse_dub_json_with_paths)),
    (DubSdl, ParserDirect(parse_dub_sdl)),
    (Gemfile, ParserDirect(parse_gemfile)),
    (RubyGemspec, ParserDirect(parse_gemspec)),
    (GoMod, ParserDirect(parse_go_mod)),
    (MavenPomXml, ParserWithPaths(parse_maven_xml_with_paths)),
    (GradleBuild, ParserDirect(parse_gradle_build)),
    (GradleSettings, ParserDirect(parse_gradle_settings)),
    (
        GradleVersionCatalogToml,
        ParserDirect(parse_gradle_version_catalog_toml),
    ),
    (SbtBuild, ParserDirect(parse_sbt_build)),
    (ClojureDepsEdn, ParserDirect(parse_clojure_deps_edn)),
    (
        LeiningenProjectClj,
        ParserDirect(parse_leiningen_project_clj),
    ),
    (MixExs, ParserDirect(parse_mix_exs)),
    (RebarConfig, ParserDirect(parse_rebar_config)),
    (GleamToml, ParserDirect(parse_gleam_toml)),
    (Opam, ParserDirect(parse_opam)),
    (DuneProject, ParserDirect(parse_dune_project)),
    (Cabal, ParserDirect(parse_cabal)),
    (CabalProject, ParserDirect(parse_cabal_project)),
    (StackYaml, ParserDirect(parse_stack_yaml)),
    (JuliaProjectToml, ParserDirect(parse_julia_project)),
    (JuliaManifestToml, ParserDirect(parse_julia_manifest)),
    (RDescription, ParserDirect(parse_r_description)),
    (RenvLock, ParserDirect(parse_renv_lock)),
    (ConanfileTxt, ParserDirect(parse_conanfile_txt)),
    (ConanfilePy, ParserDirect(parse_conanfile_py)),
    (VcpkgJson, ParserWithPaths(parse_vcpkg_json_with_paths)),
    (Cmake, ParserDirect(parse_cmake)),
    (XmakeLua, ParserDirect(parse_xmake_lua)),
    (MesonWrap, ParserDirect(parse_meson_wrap)),
    (BazelWorkspace, ParserDirect(parse_bazel_workspace)),
    (SwiftPackage, ParserDirect(parse_swift_package)),
    (ZigBuildZon, ParserDirect(parse_zig_build_zon)),
    (Nimble, ParserDirect(parse_nimble)),
    (LuaRockspec, ParserDirect(parse_luarocks_rockspec)),
    (Cpanfile, ParserDirect(parse_cpanfile)),
    (HaxelibJson, ParserWithPaths(parse_haxelib_json_with_paths)),
    (TerraformTf, ParserDirect(parse_terraform_hcl)),
    (
        HelmChartYaml,
        ParserWithPaths(parse_helm_chart_yaml_with_paths),
    ),
    (
        AnsibleGalaxyRequirementsYaml,
        ParserWithPaths(parse_ansible_galaxy_requirements_yaml_with_paths),
    ),
    (BazelModule, ParserWithPaths(parse_bazel_module_with_paths)),
    (NixFlake, ParserWithPaths(parse_nix_flake_with_paths)),
    (
        UnityProjectManifestJson,
        ParserWithPaths(parse_unity_project_manifest_json_with_paths),
    ),
    (CocoaPodsPodfile, ParserDirect(parse_cocoapods_podfile)),
    (
        NpmPackageJson,
        ParserWithPaths(parse_package_json_with_paths),
    ),
    (
        NpmPackageJson5,
        ParserWithPaths(parse_package_json_with_paths),
    ),
    (NpmPackageYaml, ParserWithPaths(parse_pnpm_yaml_with_paths)),
    (PnpmYaml, ParserWithPaths(parse_pnpm_yaml_with_paths)),
    (PythonPipfile, ParserWithPaths(parse_pipfile_with_paths)),
    (
        PythonPyprojectToml,
        ParserWithPaths(parse_pyproject_toml_with_paths),
    ),
    (PythonRequirementsTxt, ParserDirect(parse_requirements_txt)),
    (
        PubspecOverridesYaml,
        ParserWithPaths(parse_pubspec_overrides_yaml_with_paths),
    ),
    (PubspecYaml, ParserWithPaths(parse_pubspec_yaml_with_paths)),
];

pub(super) fn parse_manifest_kind(
    kind: ManifestKind,
    text: &str,
    paths: &[&str],
) -> ParsedDependencies {
    for (candidate, parser) in MANIFEST_PARSERS {
        if *candidate == kind {
            return parser.parse(text, paths);
        }
    }

    vec![]
}

fn parse_pubspec_overrides_yaml_with_paths(text: &str, paths: &[&str]) -> ParsedDependencies {
    let override_paths = if paths.is_empty() {
        &["dependency_overrides"][..]
    } else {
        paths
    };
    parse_pubspec_yaml_with_paths(text, override_paths)
}
