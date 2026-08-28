use versionlens_model::ManifestKind::{
    Cabal, DockerComposeYaml, Dockerfile, JuliaManifestToml, LuaRockspec, Nimble, Opam,
    RubyGemspec, TerraformTf, UnityProjectManifestJson, Unknown, VersionLensMultiRegistries,
};
use versionlens_model::{DocumentInput, ManifestKind};

mod content;
mod patterns;
mod tables;
mod uri;

use content::classify_content_manifest;
use patterns::{classify_early_pattern_manifest, classify_python_manifest, is_dockerfile_uri};
use tables::{
    EARLY_FILE_MANIFESTS, LATE_FILE_MANIFESTS, PUBSPEC_FILE_MANIFESTS, exact_file_manifest,
};
use uri::{SCHEMA_URI, document_uri, file_name, has_extension, is_file_uri};

pub fn classify_document(input: &DocumentInput) -> ManifestKind {
    let uri = document_uri(&input.uri);
    if let Some(kind) = classify_special_uri(uri) {
        return kind;
    }
    if !is_file_uri(uri) {
        return Unknown;
    }

    classify_unity_project_manifest(uri)
        .or_else(|| classify_early_manifest(uri))
        .or_else(|| classify_docker_manifest(&input.language_id, uri))
        .or_else(|| classify_github_actions(uri))
        .or_else(|| exact_file_manifest(uri, LATE_FILE_MANIFESTS))
        .or_else(|| classify_content_manifest(input, uri))
        .or_else(|| classify_python_manifest(&input.language_id, uri))
        .or_else(|| exact_file_manifest(uri, PUBSPEC_FILE_MANIFESTS))
        .unwrap_or(Unknown)
}

fn classify_special_uri(uri: &str) -> Option<ManifestKind> {
    (uri == SCHEMA_URI).then_some(VersionLensMultiRegistries)
}

fn classify_unity_project_manifest(uri: &str) -> Option<ManifestKind> {
    (uri.ends_with("/Packages/manifest.json") || uri.ends_with("/packages/manifest.json"))
        .then_some(UnityProjectManifestJson)
}

fn classify_early_manifest(uri: &str) -> Option<ManifestKind> {
    exact_file_manifest(uri, EARLY_FILE_MANIFESTS)
        .or_else(|| has_extension(uri, ["gemspec"]).then_some(RubyGemspec))
        .or_else(|| has_extension(uri, ["opam"]).then_some(Opam))
        .or_else(|| has_extension(uri, ["cabal"]).then_some(Cabal))
        .or_else(|| has_extension(uri, ["nimble"]).then_some(Nimble))
        .or_else(|| has_extension(uri, ["rockspec"]).then_some(LuaRockspec))
        .or_else(|| has_extension(uri, ["tf", "tofu"]).then_some(TerraformTf))
        .or_else(|| classify_julia_versioned_manifest(uri))
        .or_else(|| classify_early_pattern_manifest(uri))
}

fn classify_julia_versioned_manifest(uri: &str) -> Option<ManifestKind> {
    let name = file_name(uri)?;
    let version = name
        .strip_prefix("Manifest-v")
        .and_then(|value| value.strip_suffix(".toml"))?;
    let (major, minor) = version.split_once('.')?;
    (!major.is_empty()
        && !minor.is_empty()
        && major.bytes().all(|byte| byte.is_ascii_digit())
        && minor.bytes().all(|byte| byte.is_ascii_digit()))
    .then_some(JuliaManifestToml)
}

fn classify_docker_manifest(language_id: &str, uri: &str) -> Option<ManifestKind> {
    if language_id == "dockercompose" {
        return Some(DockerComposeYaml);
    }
    (language_id == "dockerfile" || is_dockerfile_uri(uri)).then_some(Dockerfile)
}

fn classify_github_actions(uri: &str) -> Option<ManifestKind> {
    let path = uri.strip_prefix("file:")?;
    let segments = path.split('/').collect::<Vec<_>>();
    let workflow_file = uri.rsplit('/').next().is_some_and(|name| {
        name.to_ascii_lowercase().ends_with(".yml") || name.to_ascii_lowercase().ends_with(".yaml")
    });
    let workflows = workflow_file
        && segments.windows(2).any(|window| {
            window[0].eq_ignore_ascii_case(".github") && window[1].eq_ignore_ascii_case("workflows")
        });
    let action_file = segments.windows(2).any(|window| {
        window[0].eq_ignore_ascii_case(".github") && window[1].eq_ignore_ascii_case("actions")
    }) && uri.rsplit('/').next().is_some_and(|name| {
        name.eq_ignore_ascii_case("action.yml") || name.eq_ignore_ascii_case("action.yaml")
    });
    (workflows || action_file).then_some(ManifestKind::GitHubActions)
}

#[cfg(test)]
mod tests;
