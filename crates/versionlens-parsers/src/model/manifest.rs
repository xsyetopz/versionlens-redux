use serde::{Deserialize, Serialize};

use super::Ecosystem;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ManifestKind {
    CargoToml,
    ComposerJson,
    DenoJson,
    DotnetProjectJson,
    DotnetXml,
    DockerComposeYaml,
    Dockerfile,
    DubJson,
    Gemfile,
    GoMod,
    MavenPomXml,
    NpmPackageJson,
    PnpmYaml,
    PythonPipfile,
    PythonPyprojectToml,
    PythonRequirementsTxt,
    PubspecYaml,
    VersionLensMultiRegistries,
    Unknown,
}

const MANIFEST_ECOSYSTEMS: &[(ManifestKind, Ecosystem)] = &[
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

pub fn ecosystem_for_manifest(kind: ManifestKind) -> Option<Ecosystem> {
    MANIFEST_ECOSYSTEMS
        .iter()
        .find_map(|(candidate, ecosystem)| (*candidate == kind).then_some(*ecosystem))
}
