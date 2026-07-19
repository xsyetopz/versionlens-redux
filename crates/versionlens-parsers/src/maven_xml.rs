use versionlens_model::Dependency;

mod dependency;
mod metadata;
mod nodes;
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

use dependency::collect_maven_dependencies;
use nodes::collect_nodes;

const MAVEN_DEPENDENCY_PATHS: &[&str] = &[
    "project.version",
    "project.dependencies.dependency",
    "project.dependencyManagement.dependencies.dependency",
    "project.parent",
    "project.build.plugins.plugin",
    "project.build.pluginManagement.plugins.plugin",
    "project.profiles.profile.dependencies.dependency",
    "project.profiles.profile.dependencyManagement.dependencies.dependency",
    "project.profiles.profile.build.plugins.plugin",
    "project.profiles.profile.build.pluginManagement.plugins.plugin",
];

pub(crate) fn parse_maven_xml_with_paths(text: &str, dependency_paths: &[&str]) -> Vec<Dependency> {
    let Some(nodes) = collect_nodes(text) else {
        return vec![];
    };
    let dependency_paths = selected_dependency_paths(dependency_paths);
    collect_maven_dependencies(text, &nodes, &dependency_paths)
}

fn selected_dependency_paths<'a>(dependency_paths: &'a [&'a str]) -> Vec<&'a str> {
    if dependency_paths.is_empty() {
        MAVEN_DEPENDENCY_PATHS.to_vec()
    } else {
        dependency_paths.to_vec()
    }
}

#[cfg(test)]
mod tests;
