use super::super::nodes::{XmlNode, collect_nodes, texts_from_nodes};
use super::profile::{active_profile_ids, profile_is_active};

const SETTINGS_LOCAL_REPOSITORY_PATH: &str = "settings.localRepository";
const SETTINGS_REPOSITORY_URL_PATH: &str = "settings.profiles.profile.repositories.repository.url";
const SETTINGS_PLUGIN_REPOSITORY_URL_PATH: &str =
    "settings.profiles.profile.pluginRepositories.pluginRepository.url";

pub(super) fn repository_urls_from_settings_xml(text: &str) -> Vec<String> {
    let Some(settings_xml) = settings_xml(text) else {
        return vec![];
    };
    let Some(nodes) = collect_nodes(settings_xml) else {
        return vec![];
    };

    let active_profiles = active_profile_ids(&nodes);
    let mut repositories = texts_from_nodes(&nodes, SETTINGS_LOCAL_REPOSITORY_PATH);
    repositories.extend(active_profile_url_texts(
        &nodes,
        &active_profiles,
        SETTINGS_REPOSITORY_URL_PATH,
    ));
    repositories.extend(active_profile_url_texts(
        &nodes,
        &active_profiles,
        SETTINGS_PLUGIN_REPOSITORY_URL_PATH,
    ));
    repositories
}

fn active_profile_url_texts(
    nodes: &[XmlNode],
    active_profiles: &[String],
    path: &str,
) -> Vec<String> {
    nodes
        .iter()
        .filter(|node| node.path == path && !node.text.is_empty())
        .filter(|node| profile_is_active(node, nodes, active_profiles))
        .map(|node| node.text.as_str().to_owned())
        .collect()
}

fn settings_xml(text: &str) -> Option<&str> {
    let start = text.find("<?xml")?;
    let end = text[start..].find("</settings>")? + start + "</settings>".len();
    text.get(start..end)
}
