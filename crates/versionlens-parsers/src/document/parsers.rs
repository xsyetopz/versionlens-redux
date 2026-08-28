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
use crate::sbt_build::parse_sbt_build;
use crate::swift_package::parse_swift_package;
use crate::terraform_hcl::parse_terraform_hcl;
use crate::unity_manifest::parse_unity_project_manifest_json_with_paths;
use crate::vcpkg::parse_vcpkg_json_with_paths;
use crate::zig_zon::parse_zig_build_zon;
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
    (
        ManifestKind::CargoToml,
        ParserWithPaths(parse_cargo_toml_with_paths),
    ),
    (
        ManifestKind::ComposerJson,
        ParserWithPaths(parse_composer_json_with_paths),
    ),
    (
        ManifestKind::DenoJson,
        ParserWithPaths(parse_deno_json_with_paths),
    ),
    (
        ManifestKind::DenoImportMapJson,
        ParserWithPaths(parse_deno_json_with_paths),
    ),
    (
        ManifestKind::JsrJson,
        ParserWithPaths(parse_jsr_json_with_paths),
    ),
    (
        ManifestKind::DotnetProjectJson,
        ParserWithPaths(parse_dotnet_project_json_with_paths),
    ),
    (
        ManifestKind::DotnetXml,
        ParserWithPaths(parse_dotnet_xml_with_paths),
    ),
    (
        ManifestKind::PaketDependencies,
        ParserDirect(parse_paket_dependencies),
    ),
    (
        ManifestKind::PaketReferences,
        ParserDirect(parse_paket_references),
    ),
    (
        ManifestKind::DockerComposeYaml,
        ParserDirect(parse_docker_compose_yaml),
    ),
    (ManifestKind::Dockerfile, ParserDirect(parse_dockerfile)),
    (
        ManifestKind::KustomizationYaml,
        ParserWithPaths(parse_kustomization_yaml_with_paths),
    ),
    (
        ManifestKind::DubJson,
        ParserWithPaths(parse_dub_json_with_paths),
    ),
    (ManifestKind::DubSdl, ParserDirect(parse_dub_sdl)),
    (ManifestKind::Gemfile, ParserDirect(parse_gemfile)),
    (ManifestKind::RubyGemspec, ParserDirect(parse_gemspec)),
    (ManifestKind::GoMod, ParserDirect(parse_go_mod)),
    (
        ManifestKind::MavenPomXml,
        ParserWithPaths(parse_maven_xml_with_paths),
    ),
    (ManifestKind::GradleBuild, ParserDirect(parse_gradle_build)),
    (
        ManifestKind::GradleSettings,
        ParserDirect(parse_gradle_settings),
    ),
    (
        ManifestKind::GradleVersionCatalogToml,
        ParserDirect(parse_gradle_version_catalog_toml),
    ),
    (ManifestKind::SbtBuild, ParserDirect(parse_sbt_build)),
    (
        ManifestKind::ClojureDepsEdn,
        ParserDirect(parse_clojure_deps_edn),
    ),
    (
        ManifestKind::LeiningenProjectClj,
        ParserDirect(parse_leiningen_project_clj),
    ),
    (ManifestKind::MixExs, ParserDirect(parse_mix_exs)),
    (ManifestKind::RebarConfig, ParserDirect(parse_rebar_config)),
    (ManifestKind::GleamToml, ParserDirect(parse_gleam_toml)),
    (ManifestKind::Opam, ParserDirect(parse_opam)),
    (ManifestKind::DuneProject, ParserDirect(parse_dune_project)),
    (ManifestKind::Cabal, ParserDirect(parse_cabal)),
    (
        ManifestKind::CabalProject,
        ParserDirect(parse_cabal_project),
    ),
    (ManifestKind::StackYaml, ParserDirect(parse_stack_yaml)),
    (
        ManifestKind::JuliaProjectToml,
        ParserDirect(parse_julia_project),
    ),
    (
        ManifestKind::JuliaManifestToml,
        ParserDirect(parse_julia_manifest),
    ),
    (
        ManifestKind::RDescription,
        ParserDirect(parse_r_description),
    ),
    (ManifestKind::RenvLock, ParserDirect(parse_renv_lock)),
    (
        ManifestKind::ConanfileTxt,
        ParserDirect(parse_conanfile_txt),
    ),
    (ManifestKind::ConanfilePy, ParserDirect(parse_conanfile_py)),
    (
        ManifestKind::VcpkgJson,
        ParserWithPaths(parse_vcpkg_json_with_paths),
    ),
    (ManifestKind::Cmake, ParserDirect(parse_cmake)),
    (ManifestKind::XmakeLua, ParserDirect(parse_xmake_lua)),
    (ManifestKind::MesonWrap, ParserDirect(parse_meson_wrap)),
    (
        ManifestKind::BazelWorkspace,
        ParserDirect(parse_bazel_workspace),
    ),
    (
        ManifestKind::SwiftPackage,
        ParserDirect(parse_swift_package),
    ),
    (ManifestKind::ZigBuildZon, ParserDirect(parse_zig_build_zon)),
    (ManifestKind::Nimble, ParserDirect(parse_nimble)),
    (
        ManifestKind::LuaRockspec,
        ParserDirect(parse_luarocks_rockspec),
    ),
    (ManifestKind::Cpanfile, ParserDirect(parse_cpanfile)),
    (
        ManifestKind::HaxelibJson,
        ParserWithPaths(parse_haxelib_json_with_paths),
    ),
    (ManifestKind::TerraformTf, ParserDirect(parse_terraform_hcl)),
    (
        ManifestKind::HelmChartYaml,
        ParserWithPaths(parse_helm_chart_yaml_with_paths),
    ),
    (
        ManifestKind::AnsibleGalaxyRequirementsYaml,
        ParserWithPaths(parse_ansible_galaxy_requirements_yaml_with_paths),
    ),
    (
        ManifestKind::BazelModule,
        ParserWithPaths(parse_bazel_module_with_paths),
    ),
    (
        ManifestKind::NixFlake,
        ParserWithPaths(parse_nix_flake_with_paths),
    ),
    (
        ManifestKind::UnityProjectManifestJson,
        ParserWithPaths(parse_unity_project_manifest_json_with_paths),
    ),
    (
        ManifestKind::CocoaPodsPodfile,
        ParserDirect(parse_cocoapods_podfile),
    ),
    (
        ManifestKind::NpmPackageJson,
        ParserWithPaths(parse_package_json_with_paths),
    ),
    (
        ManifestKind::NpmPackageJson5,
        ParserWithPaths(parse_package_json_with_paths),
    ),
    (
        ManifestKind::NpmPackageYaml,
        ParserWithPaths(parse_pnpm_yaml_with_paths),
    ),
    (
        ManifestKind::PnpmYaml,
        ParserWithPaths(parse_pnpm_yaml_with_paths),
    ),
    (
        ManifestKind::PythonPipfile,
        ParserWithPaths(parse_pipfile_with_paths),
    ),
    (
        ManifestKind::PythonPyprojectToml,
        ParserWithPaths(parse_pyproject_toml_with_paths),
    ),
    (
        ManifestKind::PythonRequirementsTxt,
        ParserDirect(parse_requirements_txt),
    ),
    (
        ManifestKind::PubspecOverridesYaml,
        ParserWithPaths(parse_pubspec_overrides_yaml_with_paths),
    ),
    (
        ManifestKind::PubspecYaml,
        ParserWithPaths(parse_pubspec_yaml_with_paths),
    ),
    (
        ManifestKind::GitHubActions,
        ParserDirect(parse_github_actions),
    ),
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
