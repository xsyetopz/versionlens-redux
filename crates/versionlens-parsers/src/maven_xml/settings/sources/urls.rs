use super::super::entries::MavenRepository;
use super::super::protocol::protocol_from_url;
use super::super::xml::repository_urls_from_settings_xml;

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
