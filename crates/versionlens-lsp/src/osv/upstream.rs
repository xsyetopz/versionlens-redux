//! Repositories of packages whose registry OSV has no ecosystem for, read from the registry's own
//! metadata, such as a Bazel module's `"github:owner/repo"`. OSV's GIT ecosystem files the
//! advisories of many such repositories, C and C++ libraries above all.

use tokio::task::JoinSet;

use super::Query;
use crate::extract::regex;
use crate::http::Http;

/// Each query as OSV can answer it: an upstream query renamed to its repository's URL, or `None`
/// where the registry names no GitHub repository for the package.
pub(crate) async fn resolve(http: &Http, queries: &[Query]) -> Result<Vec<Option<Query>>, String> {
    let mut tasks = JoinSet::new();
    for (index, query) in queries.iter().enumerate() {
        let (http, query) = (http.clone(), query.clone());
        tasks.spawn(async move { (index, repository(&http, query).await) });
    }
    let mut out = vec![None; queries.len()];
    while let Some(joined) = tasks.join_next().await {
        let (index, query) = joined.map_err(|e| e.to_string())?;
        out[index] = query?;
    }
    Ok(out)
}

async fn repository(http: &Http, query: Query) -> Result<Option<Query>, String> {
    if query.upstream.is_empty() {
        return Ok(Some(query));
    }
    let text = match http.text(&query.name, "*/*").await {
        Err(e) if e.contains("HTTP 404") => return Ok(None),
        fetched => fetched?,
    };
    // Repository tags carry no package revision, such as LuaRocks' `-0` or Bazel's `.bcr.1`.
    let version = query.version.split(".bcr.").next().unwrap_or_default();
    let version = match version.rsplit_once('-') {
        Some((tag, revision)) if revision.bytes().all(|b| b.is_ascii_digit()) => tag,
        _ => version,
    }
    .to_owned();
    Ok(found(query.upstream, &text).map(|name| Query {
        name,
        version,
        upstream: "",
        ..query
    }))
}

/// The URL of the GitHub repository the pattern finds in a registry document.
pub(super) fn found(pattern: &'static str, text: &str) -> Option<String> {
    let caps = regex(pattern).captures_iter(text).next()?;
    let group = |name| caps.get_group_by_name(name).map(|s| &text[s.range()]);
    let repo = group("repo")?;
    Some(format!(
        "https://github.com/{}/{}",
        group("owner")?,
        repo.strip_suffix(".git").unwrap_or(repo)
    ))
}

#[cfg(test)]
mod tests;
