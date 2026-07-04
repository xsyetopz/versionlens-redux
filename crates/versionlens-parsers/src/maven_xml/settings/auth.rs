use std::collections::HashMap;

use base64::{Engine, engine::general_purpose::STANDARD};

use super::super::nodes::{XmlNode, child_named, collect_nodes, direct_children};
use super::model::{MavenAuthEntry, MavenMirror, MavenNamedRepository};

pub fn parse_maven_settings_auth_entries(text: &str) -> Vec<MavenAuthEntry> {
    let Some(nodes) = collect_nodes(text) else {
        return Vec::new();
    };
    let credentials = server_credentials(&nodes);

    repository_sources(&nodes)
        .into_iter()
        .chain(mirror_sources(&nodes))
        .filter_map(|(id, url)| auth_entry(id, url, &credentials))
        .collect()
}

pub fn parse_maven_settings_repository_urls(text: &str) -> Vec<String> {
    parse_maven_settings_repositories(text)
        .into_iter()
        .map(|repository| repository.url)
        .collect()
}

pub fn parse_maven_settings_repositories(text: &str) -> Vec<MavenNamedRepository> {
    let Some(nodes) = collect_nodes(text) else {
        return Vec::new();
    };

    local_repository_sources(&nodes)
        .into_iter()
        .chain(repository_sources(&nodes))
        .map(|(id, url)| MavenNamedRepository { id, url })
        .collect()
}

pub fn parse_maven_settings_mirror_urls(text: &str) -> Vec<String> {
    parse_maven_settings_mirrors(text)
        .into_iter()
        .filter(|mirror| mirror.mirror_of == "*")
        .map(|mirror| mirror.url)
        .collect()
}

pub fn parse_maven_settings_mirrors(text: &str) -> Vec<MavenMirror> {
    let Some(nodes) = collect_nodes(text) else {
        return Vec::new();
    };

    nodes
        .iter()
        .filter(|node| node.path == "settings.mirrors.mirror")
        .filter_map(|node| mirror(node, &nodes))
        .collect()
}

fn mirror_sources(nodes: &[XmlNode]) -> Vec<(String, String)> {
    nodes
        .iter()
        .filter(|node| node.path == "settings.mirrors.mirror")
        .filter_map(|node| mirror_source(node, nodes))
        .collect()
}

fn mirror_source(node: &XmlNode, nodes: &[XmlNode]) -> Option<(String, String)> {
    let mirror = mirror(node, nodes)?;
    Some((mirror.id, mirror.url))
}

fn mirror(node: &XmlNode, nodes: &[XmlNode]) -> Option<MavenMirror> {
    let children = direct_children(node, nodes);
    Some(MavenMirror {
        id: child_named(&children, "id")?.text.as_str().to_owned(),
        mirror_of: child_named(&children, "mirrorOf")?.text.as_str().to_owned(),
        url: child_named(&children, "url")?.text.as_str().to_owned(),
    })
}

fn repository_sources(nodes: &[XmlNode]) -> Vec<(String, String)> {
    nodes
        .iter()
        .filter(|node| node.path == "settings.profiles.profile.repositories.repository")
        .filter_map(|node| repository_source(node, nodes))
        .collect()
}

fn local_repository_sources(nodes: &[XmlNode]) -> Vec<(String, String)> {
    nodes
        .iter()
        .filter(|node| node.path == "settings.localRepository")
        .map(|node| ("local".to_owned(), node.text.as_str().to_owned()))
        .filter(|(_, url)| !url.is_empty())
        .collect()
}

fn repository_source(node: &XmlNode, nodes: &[XmlNode]) -> Option<(String, String)> {
    let children = direct_children(node, nodes);
    let id = child_named(&children, "id")?.text.as_str().to_owned();
    let url = child_named(&children, "url")?.text.as_str().to_owned();
    Some((id, url))
}

fn server_credentials(nodes: &[XmlNode]) -> HashMap<String, (String, String)> {
    nodes
        .iter()
        .filter(|node| node.path == "settings.servers.server")
        .filter_map(|node| server_credential(node, nodes))
        .collect()
}

fn server_credential(node: &XmlNode, nodes: &[XmlNode]) -> Option<(String, (String, String))> {
    let children = direct_children(node, nodes);
    let id = child_named(&children, "id")?.text.as_str().to_owned();
    let username = child_named(&children, "username")?.text.as_str().to_owned();
    let password = child_named(&children, "password")?.text.as_str().to_owned();
    Some((id, (username, password)))
}

fn auth_entry(
    id: String,
    registry: String,
    credentials: &HashMap<String, (String, String)>,
) -> Option<MavenAuthEntry> {
    let (username, password) = credentials.get(&id)?;
    let token = STANDARD.encode(format!("{username}:{password}"));

    Some(MavenAuthEntry {
        registry,
        header_value: format!("Basic {token}"),
    })
}
