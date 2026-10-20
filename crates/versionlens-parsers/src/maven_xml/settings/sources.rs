use super::entries::MavenRepository;
use super::xml::repository_urls_from_settings_xml;

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

const DEFAULT_MAVEN_REPOSITORY: &str = "https://repo.maven.apache.org/maven2/";

pub fn parse_maven_effective_settings_repositories(text: &str) -> Vec<String> {
    repository_urls_from_settings_xml(text)
}

pub fn parse_maven_effective_settings_https_repositories(text: &str) -> Vec<String> {
    settings_repository_urls_with_default(text)
        .into_iter()
        .filter(|repository| protocol_from_url(repository) == "https:")
        .collect()
}

pub(super) fn settings_repository_urls_with_default(text: &str) -> Vec<String> {
    let mut repositories = parse_maven_effective_settings_repositories(text);
    if repositories.is_empty() {
        repositories.push(DEFAULT_MAVEN_REPOSITORY.to_owned());
    }
    repositories
}

pub fn extract_maven_repository_urls(repositories: &[MavenRepository]) -> Vec<String> {
    repositories
        .iter()
        .map(|repository| repository.url.as_str().to_owned())
        .collect()
}

fn protocol_from_url(url: &str) -> &str {
    url.find(':')
        .map(|index| &url[..=index])
        .filter(|protocol| matches!(*protocol, "http:" | "https:"))
        .unwrap_or("file:")
}
