use super::super::nodes::{collect_nodes, texts_from_nodes};

const SETTINGS_LOCAL_REPOSITORY_PATH: &str = "settings.localRepository";
const SETTINGS_REPOSITORY_URL_PATH: &str = "settings.profiles.profile.repositories.repository.url";

pub(super) fn repository_urls_from_settings_xml(text: &str) -> Vec<String> {
    let Some(settings_xml) = settings_xml(text) else {
        return Vec::new();
    };
    let Some(nodes) = collect_nodes(settings_xml) else {
        return Vec::new();
    };

    let mut repositories = texts_from_nodes(&nodes, SETTINGS_LOCAL_REPOSITORY_PATH);
    repositories.extend(texts_from_nodes(&nodes, SETTINGS_REPOSITORY_URL_PATH));
    repositories
}

fn settings_xml(text: &str) -> Option<&str> {
    let start = text.find("<?xml")?;
    let end = text[start..].find("</settings>")? + start + "</settings>".len();
    text.get(start..end)
}
