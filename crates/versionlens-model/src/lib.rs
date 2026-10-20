mod document;
mod ecosystem;
mod edit;
mod github;
mod manifest;
mod position;
mod versions;

pub use document::{
    CanonicalReference, Dependency, DocumentInput, VersionableKind, is_npm_dist_tag_requirement,
    registry_alias_requirement,
};
pub use ecosystem::{
    Ecosystem, all_ecosystems, ecosystem_config_namespace, ecosystem_from_config_name,
    ecosystem_provider_id,
};
pub use edit::{
    DocumentEditPlan, DocumentSnapshot, TextEdit, WorkspaceEditPlan, document_text_hash,
};
pub use github::GithubRepository;
pub use manifest::ManifestKind;
pub use position::{Position, Range};
pub use versions::{
    ProjectVersionBump, UpdateLevel, VersionDialect, build_variants, compare_numeric_segments,
    compare_numeric_text, compare_versions, compare_versions_for_dialect,
    composer_requirement_is_parseable, is_build_update, is_dotnet_requirement_parseable, is_newer,
    is_prerelease_project_version, is_update_available, is_update_available_for_dialect,
    latest_stable, latest_version, latest_version_for_dialect, latest_version_with_prerelease_tags,
    next_project_version, normalized_version, normalized_version_for_dialect, numeric_segments,
    parse_semver, requirement_has_empty_comparator_intersection, requirement_is_parseable,
    requirement_is_parseable_for_dialect, requirement_satisfies_latest,
    requirement_satisfies_latest_for_dialect, strip_version_prefix, update_level,
    version_tag_parts,
};

pub fn strip_matching_quotes(value: &str) -> &str {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .or_else(|| {
            value
                .strip_prefix('\'')
                .and_then(|value| value.strip_suffix('\''))
        })
        .unwrap_or(value)
}
