use crate::cargo_toml::parse_cargo_toml_with_paths;
use crate::dotnet_xml::parse_dotnet_xml_with_paths;
use crate::json_manifest::{
    parse_composer_json_with_paths, parse_deno_json_with_paths,
    parse_dotnet_project_json_with_paths, parse_dub_json_with_paths, parse_package_json_with_paths,
};
use crate::maven_xml::parse_maven_xml_with_paths;
use crate::model::{Dependency, ManifestKind};
use crate::pnpm_yaml::parse_pnpm_yaml_with_paths;
use crate::pubspec_yaml::parse_pubspec_yaml_with_paths;
use crate::pyproject_toml::{parse_pipfile_with_paths, parse_pyproject_toml_with_paths};

mod direct;

use direct::{
    parse_docker_compose_document, parse_dockerfile_document, parse_gemfile_document,
    parse_go_mod_document, parse_requirements_document,
};

type ManifestParser = fn(&str, &[&str]) -> Vec<Dependency>;

const MANIFEST_PARSERS: &[(ManifestKind, ManifestParser)] = &[
    (ManifestKind::CargoToml, parse_cargo_toml_with_paths),
    (ManifestKind::ComposerJson, parse_composer_json_with_paths),
    (ManifestKind::DenoJson, parse_deno_json_with_paths),
    (
        ManifestKind::DotnetProjectJson,
        parse_dotnet_project_json_with_paths,
    ),
    (ManifestKind::DotnetXml, parse_dotnet_xml_with_paths),
    (
        ManifestKind::DockerComposeYaml,
        parse_docker_compose_document,
    ),
    (ManifestKind::Dockerfile, parse_dockerfile_document),
    (ManifestKind::DubJson, parse_dub_json_with_paths),
    (ManifestKind::Gemfile, parse_gemfile_document),
    (ManifestKind::GoMod, parse_go_mod_document),
    (ManifestKind::MavenPomXml, parse_maven_xml_with_paths),
    (ManifestKind::NpmPackageJson, parse_package_json_with_paths),
    (ManifestKind::PnpmYaml, parse_pnpm_yaml_with_paths),
    (ManifestKind::PythonPipfile, parse_pipfile_with_paths),
    (
        ManifestKind::PythonPyprojectToml,
        parse_pyproject_toml_with_paths,
    ),
    (
        ManifestKind::PythonRequirementsTxt,
        parse_requirements_document,
    ),
    (ManifestKind::PubspecYaml, parse_pubspec_yaml_with_paths),
];

pub(super) fn parse_manifest_kind(
    kind: ManifestKind,
    text: &str,
    paths: &[&str],
) -> Vec<Dependency> {
    for (candidate, parser) in MANIFEST_PARSERS {
        if *candidate == kind {
            return parser(text, paths);
        }
    }

    Vec::new()
}
