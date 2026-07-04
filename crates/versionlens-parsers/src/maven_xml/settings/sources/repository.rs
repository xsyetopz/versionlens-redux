use super::urls::settings_repository_urls_with_default;
use crate::maven_xml::settings::{model::MavenRepository, protocol::protocol_from_url};

pub fn parse_maven_effective_settings_repository_sources(text: &str) -> Vec<MavenRepository> {
    settings_repository_urls_with_default(text)
        .into_iter()
        .map(maven_repository_from_url)
        .collect()
}

pub fn parse_maven_effective_settings_https_repository_sources(text: &str) -> Vec<MavenRepository> {
    parse_maven_effective_settings_repository_sources(text)
        .into_iter()
        .filter(|source| source.protocol == "https:")
        .collect()
}

fn maven_repository_from_url(url: String) -> MavenRepository {
    MavenRepository {
        protocol: protocol_from_url(&url).to_owned(),
        url,
    }
}
