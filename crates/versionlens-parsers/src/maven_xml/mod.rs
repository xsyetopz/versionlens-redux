use versionlens_model::Dependency;

mod dependency;
mod metadata;
mod repositories;
mod settings;

pub use metadata::parse_maven_metadata_versions;
pub use repositories::{parse_maven_pom_repositories, parse_maven_pom_repository_urls};
pub use settings::{
    MavenAuthEntry, MavenMirror, MavenNamedRepository, MavenRepository,
    extract_maven_repository_urls, parse_maven_effective_settings_https_repositories,
    parse_maven_effective_settings_https_repository_sources,
    parse_maven_effective_settings_repositories, parse_maven_effective_settings_repository_sources,
    parse_maven_settings_auth_entries, parse_maven_settings_mirror_urls,
    parse_maven_settings_mirrors, parse_maven_settings_repositories,
    parse_maven_settings_repository_urls,
};

use crate::manifest::adapter::xml::collect_nodes;
use dependency::collect_maven_dependencies;

pub(crate) fn parse_maven_xml_with_paths(text: &str, dependency_paths: &[&str]) -> Vec<Dependency> {
    let Some(nodes) = collect_nodes(text) else {
        return vec![];
    };
    collect_maven_dependencies(text, &nodes, &dependency_paths)
}

#[cfg(test)]
mod tests;
