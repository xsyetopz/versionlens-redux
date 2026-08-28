use serde as _;
use versionlens_model::GithubRepository;

#[test]
fn validates_and_encodes_distinct_path_components() {
    let repository = GithubRepository::parse("octokit/core.js").unwrap();
    assert_eq!(repository.to_string(), "octokit/core.js");
    assert_eq!(
        repository.api_url("https://api.github.com/repos", "/tags"),
        "https://api.github.com/repos/octokit/core.js/tags"
    );
}

#[test]
fn rejects_path_traversal_and_reserved_input() {
    for value in [
        "../repo",
        "owner/..",
        "owner/repo?query",
        "owner/repo#fragment",
        "owner/re%2Fpo",
        "owner/repo name",
        "owner/repo\n",
        "actions/checkout@v4",
        "owner/repo/extra",
    ] {
        assert!(GithubRepository::parse(value).is_none(), "{value}");
    }
}
