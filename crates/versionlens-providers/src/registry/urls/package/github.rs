use versionlens_model::GithubRepository;

pub(crate) fn github_registry_url(name: &str) -> String {
    GithubRepository::parse(name).map_or_else(String::new, |repository| repository.tags_url())
}

pub(crate) fn github_registry_url_with_base(base: &str, name: &str) -> String {
    GithubRepository::parse(name)
        .map_or_else(String::new, |repository| repository.api_url(base, "/tags"))
}
