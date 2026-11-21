//! Known vulnerabilities from OSV, asked for a whole document in one batch request.

use serde_json::{Value, json};

use crate::http::Http;

const BATCH: usize = 1000;

/// A package at a version, in an OSV ecosystem.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct Query {
    pub(crate) ecosystem: &'static str,
    pub(crate) name: String,
    pub(crate) version: String,
}

/// The vulnerability ids affecting each query, in order.
///
/// # Errors
/// Reports a failed request or an unexpected response.
pub(crate) async fn query(http: &Http, queries: &[Query]) -> Result<Vec<Vec<String>>, String> {
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
