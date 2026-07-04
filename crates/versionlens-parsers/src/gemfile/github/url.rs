pub(super) fn github_api_url(repo: &str, path: &str) -> String {
    format!("https://api.github.com/repos/{repo}/{path}")
}
