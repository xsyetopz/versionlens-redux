//! Advisories from the GitLab Advisory Database (MIT licensed), read through GitLab's GraphQL API:
//! one YAML file per advisory under the package's slug, such as `conan/openssl/CVE-2020-1967.yml`.

use serde_json::{Value, json};

use super::Query;
use crate::flatten::{Format, flatten};
use crate::http::Http;
use crate::version::{Bare, Req, Version};

/// The pseudo-ecosystem whose queries go to the GitLab Advisory Database; names are package slugs.
pub(crate) const GITLAB: &str = "GitLab";

const API: &str = "https://gitlab.com/api/graphql";

const LIST: &str = r#"query($path: String!, $after: String) { project(fullPath: "gitlab-org/advisories-community") {
  repository { tree(path: $path, ref: "main") { blobs(first: 100, after: $after) {
    nodes { path } pageInfo { hasNextPage endCursor } } } } } }"#;

const READ: &str = r#"query($paths: [String!]!) { project(fullPath: "gitlab-org/advisories-community") {
  repository { blobs(ref: "main", paths: $paths) { nodes { rawBlob } } } } }"#;

async fn graphql(http: &Http, query: &str, variables: Value) -> Result<Value, String> {
    let body = json!({ "query": query, "variables": variables }).to_string();
    let response: Value = serde_json::from_str(&http.post(API, body).await?)
        .map_err(|e| format!("GitLab: invalid JSON: {e}"))?;
    match response.pointer("/errors/0/message") {
        Some(error) => Err(format!("GitLab: {error}")),
        None => Ok(response),
    }
}

/// The advisory ids whose affected range holds the query's version.
pub(super) async fn ids(http: &Http, query: &Query) -> Result<Vec<String>, String> {
    let Some(version) = Version::parse(&query.version) else {
        return Ok(Vec::new());
    };
    let mut paths = Vec::new();
    let mut after = Value::Null;
    loop {
        let variables = json!({ "path": query.name, "after": after });
        let page = graphql(http, LIST, variables).await?;
        let Some(blobs) = page.pointer("/data/project/repository/tree/blobs") else {
            break;
        };
        let nodes = blobs.pointer("/nodes").and_then(Value::as_array);
        paths.extend(
            nodes
                .into_iter()
                .flatten()
                .filter_map(|n| n.get("path")?.as_str())
                .map(str::to_owned),
        );
        after = blobs
            .pointer("/pageInfo/endCursor")
            .cloned()
            .unwrap_or_default();
        if blobs.pointer("/pageInfo/hasNextPage") != Some(&Value::Bool(true)) {
            break;
        }
    }
    let mut ids = Vec::new();
    for chunk in paths.chunks(100) {
        let read = graphql(http, READ, json!({ "paths": chunk })).await?;
        let nodes = read
            .pointer("/data/project/repository/blobs/nodes")
            .and_then(Value::as_array);
        for raw in nodes
            .into_iter()
            .flatten()
            .filter_map(|n| n.get("rawBlob")?.as_str())
        {
            ids.extend(affecting(raw, &version));
        }
    }
    ids.sort();
    ids.dedup();
    Ok(ids)
}

/// The advisory's identifier when its `affected_range` holds the version.
pub(super) fn affecting(advisory: &str, version: &Version) -> Option<String> {
    let leaves = flatten(Format::Yaml, advisory);
    let field = |key: &str| {
        leaves
            .iter()
            .find(|l| l.path == [key])
            .map(|l| l.value.as_str())
    };
    let range = Req::parse(field("affected_range")?, Bare::Exact)?;
    range
        .matches(version)
        .then(|| field("identifier").map(str::to_owned))?
}

#[cfg(test)]
mod tests;
