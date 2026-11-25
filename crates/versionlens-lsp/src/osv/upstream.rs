//! Repositories of packages whose registry OSV has no ecosystem for, read from the registry's own
//! metadata, such as a Bazel module's `"github:owner/repo"`. OSV's GIT ecosystem files the
//! advisories of many such repositories, C and C++ libraries above all.

use tokio::task::JoinSet;

use super::Query;
use crate::http::Http;
use crate::registry::upstream_of;

/// Each query as OSV can answer it: an upstream query renamed to its repository's URL, or `None`
/// where the registry names no repository for the package.
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
    Ok(
        found(query.upstream, &query.package, &text).map(|name| Query {
            name,
            version,
            upstream: "",
            ..query
        }),
    )
}

/// The URL of the repository the pattern finds in a registry document about `package`: on the
/// host it captures, or on GitHub.
pub(super) fn found(pattern: &str, package: &str, text: &str) -> Option<String> {
    let host = if pattern.contains("(?P<host>") {
        ""
    } else {
        "github.com/"
    };
    upstream_of(pattern, package, text).map(|repo| format!("https://{host}{repo}"))
}

#[cfg(test)]
mod tests;
