use super::nodes::{XmlNode, child_named, collect_nodes, direct_children, texts_from_nodes};
use super::settings::MavenNamedRepository;

const PROJECT_REPOSITORY_PATHS: &[&str] = &[
    "project.repositories.repository",
    "project.profiles.profile.repositories.repository",
];

const PROJECT_REPOSITORY_URL_PATHS: &[&str] = &[
    "project.repositories.repository.url",
    "project.profiles.profile.repositories.repository.url",
];

pub fn parse_maven_pom_repository_urls(text: &str) -> Vec<String> {
    collect_nodes(text)
        .map(|nodes| {
            PROJECT_REPOSITORY_URL_PATHS
                .iter()
                .flat_map(|path| texts_from_nodes(&nodes, path))
                .collect()
        })
        .unwrap_or_default()
}

pub fn parse_maven_pom_repositories(text: &str) -> Vec<MavenNamedRepository> {
    collect_nodes(text)
        .map(|nodes| {
            PROJECT_REPOSITORY_PATHS
                .iter()
                .flat_map(|path| repositories_from_nodes(&nodes, path))
                .collect()
        })
        .unwrap_or_default()
}

fn repositories_from_nodes(nodes: &[XmlNode], path: &str) -> Vec<MavenNamedRepository> {
    nodes
        .iter()
        .filter(|node| node.path == path)
        .filter_map(|node| repository_from_node(node, nodes))
        .collect()
}

fn repository_from_node(node: &XmlNode, nodes: &[XmlNode]) -> Option<MavenNamedRepository> {
    let children = direct_children(node, nodes);
    Some(MavenNamedRepository {
        id: child_named(&children, "id")
            .map(|node| node.text.as_str())
            .unwrap_or_default()
            .to_owned(),
        url: child_named(&children, "url")?.text.as_str().to_owned(),
    })
}
