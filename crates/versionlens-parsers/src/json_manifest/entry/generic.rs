use crate::json_manifest::parse::parse_json_manifest;
use versionlens_model::Dependency;
use versionlens_model::Ecosystem;
use versionlens_model::Ecosystem::{Composer, Dotnet, Dub};

type JsonManifestDependencies = Vec<Dependency>;

pub(crate) fn parse_composer_json_with_paths(
    text: &str,
    paths: &[&str],
) -> JsonManifestDependencies {
    parse_manifest_with_paths(text, paths, Composer)
}

pub(crate) fn parse_dotnet_project_json_with_paths(
    text: &str,
    paths: &[&str],
) -> JsonManifestDependencies {
    parse_manifest_with_paths(text, paths, Dotnet)
}

pub(crate) fn parse_dub_json_with_paths(text: &str, paths: &[&str]) -> JsonManifestDependencies {
    parse_manifest_with_paths(text, paths, Dub)
}

pub(super) fn parse_manifest_with_paths(
    text: &str,
    paths: &[&str],
    ecosystem: Ecosystem,
) -> JsonManifestDependencies {
    parse_json_manifest(text, paths, ecosystem).unwrap_or_default()
}
