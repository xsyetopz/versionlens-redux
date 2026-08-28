mod document;
mod ecosystem;
mod edit;
mod github;
mod manifest;
mod position;
mod range;

pub use document::{
    Dependency, DocumentInput, is_npm_dist_tag_requirement, registry_alias_requirement,
};
pub use ecosystem::{
    ALL_ECOSYSTEMS, Ecosystem, ecosystem_config_namespace, ecosystem_from_config_name,
    ecosystem_provider_id,
};
pub use edit::TextEdit;
pub use github::GithubRepository;
pub use manifest::{ManifestKind, ecosystem_for_manifest, provider_name_for_manifest};
pub use position::Position;
pub use range::Range;

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
