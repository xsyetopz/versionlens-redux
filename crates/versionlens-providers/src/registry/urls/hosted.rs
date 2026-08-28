use versionlens_model::GithubRepository;

pub(super) fn github_tags_url(name: &str) -> Option<String> {
    if name.starts_with('@') {
        return None;
    }
    GithubRepository::parse(name).map(|repository| repository.tags_url())
}
