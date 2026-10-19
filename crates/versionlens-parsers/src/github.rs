use versionlens_model::GithubRepository;

const GITHUB_TAGS_BASE: &str = "https://api.github.com/repos";

pub(crate) fn repository_from_url<'a>(url: &'a str, prefixes: &[&str]) -> Option<&'a str> {
    let rest = prefixes
        .iter()
        .find_map(|prefix| url.strip_prefix(prefix))?;
    let repository = rest
        .split_once("/archive/")
        .map_or(rest, |(repository, _)| repository);
    let repository = repository
        .split_once("/releases/")
        .map_or(repository, |(repository, _)| repository)
        .trim_end_matches('/')
        .trim_end_matches(".git");
    let (owner, name) = repository.split_once('/')?;
    (repository.matches('/').count() == 1
        && !owner.is_empty()
        && !name.is_empty()
        && GithubRepository::parse(repository).is_some())
    .then_some(repository)
}

pub(crate) fn repository_from_path(path: &str) -> Option<&str> {
    repository_from_url(path, &[""])
}

pub(crate) fn tags_url(repository: &str) -> Option<String> {
    GithubRepository::parse(repository)
        .map(|repository| repository.api_url(GITHUB_TAGS_BASE, "/tags"))
}
