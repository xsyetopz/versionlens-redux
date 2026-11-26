//! The host of the git remote a workspace was cloned from. A GitLab pipeline names its own
//! instance as `$CI_SERVER_FQDN`, which GitLab sets to the instance's host and port
//! (<https://docs.gitlab.com/ci/variables/predefined_variables/>), and a pipeline's repository is a project
//! on that instance, so the remote's host stands in for the variable outside CI.

use std::path::{Path, PathBuf};

/// The host, with any port, of the `origin` remote of the repository holding `dir`, or of its
/// first remote where it has no `origin`.
pub(crate) fn remote_host(dir: &Path) -> Option<String> {
    let config = std::fs::read_to_string(config_of(dir)?).ok()?;
    let urls = remote_urls(&config);
    let url = urls
        .iter()
        .find(|(name, _)| name == "origin")
        .or_else(|| urls.first())?;
    host(&url.1)
}

/// The `config` of the repository above `dir`: `.git/config`, or for a worktree or submodule,
/// whose `.git` is a file `gitdir: <path>`, the config of the repository that path's `commondir`
/// names (git-scm.com/docs/gitrepository-layout).
fn config_of(dir: &Path) -> Option<PathBuf> {
    let dot = dir
        .ancestors()
        .map(|d| d.join(".git"))
        .find(|d| d.exists())?;
    if dot.is_dir() {
        return Some(dot.join("config"));
    }
    let text = std::fs::read_to_string(&dot).ok()?;
    let gitdir = text.strip_prefix("gitdir:")?.trim();
    let gitdir = dot.parent()?.join(gitdir);
    let common = std::fs::read_to_string(gitdir.join("commondir"))
        .map(|c| gitdir.join(c.trim()))
        .unwrap_or(gitdir);
    Some(common.join("config"))
}

/// Each `[remote "name"]` section's `url`, in file order (git-scm.com/docs/git-config, "Syntax").
fn remote_urls(config: &str) -> Vec<(String, String)> {
    let mut remote = None;
    let mut urls = Vec::new();
    for line in config.lines().map(str::trim) {
        if line.starts_with('[') {
            remote = line
                .strip_prefix("[remote \"")
                .and_then(|r| r.strip_suffix("\"]"))
                .map(str::to_owned);
        } else if let (Some(name), Some((key, value))) = (&remote, line.split_once('='))
            && key.trim().eq_ignore_ascii_case("url")
        {
            urls.push((name.clone(), value.trim().trim_matches('"').to_owned()));
        }
    }
    urls
}

/// The web host of a remote URL: `https://host:port/path` keeps its port, while an SSH remote,
/// `git@host:path` or `ssh://git@host:port/path`, gives its host alone, as its port is SSH's.
fn host(url: &str) -> Option<String> {
    if let Some((scheme, rest)) = url.split_once("://") {
        let authority = rest.split('/').next()?;
        let authority = authority.rsplit('@').next()?;
        let host = if scheme.starts_with("http") {
            authority
        } else {
            authority.split(':').next()?
        };
        return (!host.is_empty()).then(|| host.to_ascii_lowercase());
    }
    // The scp-like form `[user@]host:path` (git-scm.com/docs/git-clone, "GIT URLS").
    let (authority, _) = url.split_once(':')?;
    let host = authority.rsplit('@').next()?;
    (!host.is_empty() && !host.contains('/')).then(|| host.to_ascii_lowercase())
}

#[cfg(test)]
mod tests;
