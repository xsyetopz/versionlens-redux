//! Known vulnerabilities from OSV, asked for a whole document in one batch request. Two sources
//! cover registries OSV has no ecosystem for: Artifact Hub's image scans for Helm charts and the
//! GitLab Advisory Database for Conan.

use serde_json::{Value, json};

use crate::http::Http;
use crate::registry::{self, Registry};

const BATCH: usize = 1000;

mod artifact_hub;
mod gitlab;
pub(crate) mod upstream;

#[cfg(test)]
mod tests;

pub(crate) use artifact_hub::ARTIFACT_HUB;
pub(crate) use gitlab::GITLAB;

/// A package at a version, in an OSV ecosystem.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct Query {
    pub(crate) ecosystem: &'static str,
    pub(crate) name: String,
    pub(crate) version: String,
    /// The registry's [`crate::registry::Registry::upstream`]: where set, `name` is the URL of a
    /// document that names the repository to ask about.
    pub(crate) upstream: &'static str,
    /// The dependency's own name, which `{name}` in `upstream` stands for.
    pub(crate) package: String,
}

/// The queries for a package at a version: one in the registry's own ecosystem, and one in each
/// further ecosystem its `also` names, which takes the name as written and no upstream document.
pub(crate) fn queries(
    registry: &Registry,
    package: &str,
    source: Option<&str>,
    version: &str,
) -> Vec<Query> {
    let own = registry
        .osv
        .map(|e| (e, registry.osv_name, registry.upstream));
    let also = registry.also.iter().map(|&(e, name)| (e, name, ""));
    own.into_iter()
        .chain(also)
        .filter_map(|(ecosystem, template, upstream)| {
            let name = registry::expand(template, package, source).ok()?;
            Some(Query {
                ecosystem,
                name: name.replace("{version}", version),
                version: version.to_owned(),
                upstream,
                package: package.to_owned(),
            })
        })
        .collect()
}

/// The vulnerability ids affecting each query, in order.
///
/// # Errors
/// Reports a failed request or an unexpected response.
pub(crate) async fn query(http: &Http, queries: &[Query]) -> Result<Vec<Vec<String>>, String> {
    ids(http, &upstream::resolve(http, queries).await?).await
}

/// The advisory ids for queries whose upstream repository is already resolved; a `None`, a
/// package whose registry names no repository, has none.
pub(crate) async fn ids(http: &Http, asked: &[Option<Query>]) -> Result<Vec<Vec<String>>, String> {
    let osv: Vec<&Query> = asked
        .iter()
        .flatten()
        .filter(|q| ![ARTIFACT_HUB, GITLAB].contains(&q.ecosystem))
        .collect();
    let mut osv_ids = batch(http, &osv).await?.into_iter();
    let mut out = Vec::with_capacity(asked.len());
    for query in asked {
        out.push(match query {
            None => Vec::new(),
            Some(q) if q.ecosystem == ARTIFACT_HUB => artifact_hub::ids(http, q).await?,
            Some(q) if q.ecosystem == GITLAB => gitlab::ids(http, q).await?,
            Some(_) => osv_ids.next().unwrap_or_default(),
        });
    }
    Ok(out)
}

async fn batch(http: &Http, queries: &[&Query]) -> Result<Vec<Vec<String>>, String> {
    let mut out = Vec::with_capacity(queries.len());
    for chunk in queries.chunks(BATCH) {
        let body = json!({ "queries": chunk.iter().map(|q| json!({
            "package": { "name": q.name, "ecosystem": q.ecosystem },
            "version": q.version,
        })).collect::<Vec<_>>() });
        let text = http
            .post("https://api.osv.dev/v1/querybatch", body.to_string())
            .await?;
        let response: Value =
            serde_json::from_str(&text).map_err(|e| format!("OSV: invalid JSON: {e}"))?;
        let results = response
            .get("results")
            .and_then(Value::as_array)
            .ok_or("OSV: response has no results")?;
        if results.len() != chunk.len() {
            return Err(format!(
                "OSV: {} results for {} queries",
                results.len(),
                chunk.len()
            ));
        }
        out.extend(results.iter().map(|r| {
            r.get("vulns")
                .and_then(Value::as_array)
                .map_or_default(|vulns| {
                    vulns
                        .iter()
                        .filter_map(|v| v.get("id")?.as_str().map(str::to_owned))
                        .collect()
                })
        }));
    }
    Ok(out)
}
