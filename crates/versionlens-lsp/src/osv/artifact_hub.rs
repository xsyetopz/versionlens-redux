//! Helm chart advisories from Artifact Hub, which scans every image a chart version deploys with
//! Trivy and publishes the report.

use serde_json::Value;

use super::Query;
use crate::http::Http;

/// The pseudo-ecosystem whose queries go to Artifact Hub; its names read `repository-url chart`.
pub(crate) const ARTIFACT_HUB: &str = "Artifact Hub";

const API: &str = "https://artifacthub.io/api/v1";

/// Fetches an Artifact Hub document; `None` where Artifact Hub lists no such repository,
/// chart, or report.
async fn hub(http: &Http, path: &str) -> Result<Option<Value>, String> {
    match http
        .text(&format!("{API}/{path}"), "application/json")
        .await
    {
        Err(e) if e.contains("HTTP 404") => Ok(None),
        fetched => serde_json::from_str(&fetched?)
            .map(Some)
            .map_err(|e| format!("Artifact Hub: invalid JSON: {e}")),
    }
}

/// The CVEs Artifact Hub's Trivy scan found in the images a chart version deploys.
pub(super) async fn ids(http: &Http, query: &Query) -> Result<Vec<String>, String> {
    let Some((source, chart)) = query.name.split_once(' ') else {
        return Ok(Vec::new());
    };
    let version = &query.version;
    let repos = hub(http, &format!("repositories/search?url={source}&limit=1")).await?;
    let Some(repo) = repos.as_ref().and_then(|r| r.pointer("/0/name")?.as_str()) else {
        return Ok(Vec::new());
    };
    let package = hub(http, &format!("packages/helm/{repo}/{chart}/{version}")).await?;
    let Some(id) = package.as_ref().and_then(|p| p.get("package_id")?.as_str()) else {
        return Ok(Vec::new());
    };
    let report = hub(http, &format!("packages/{id}/{version}/security-report")).await?;
    let mut ids: Vec<String> = report
        .as_ref()
        .and_then(Value::as_object)
        .into_iter()
        .flat_map(|images| images.values())
        .filter_map(|image| image.get("Results")?.as_array())
        .flatten()
        .filter_map(|result| result.get("Vulnerabilities")?.as_array())
        .flatten()
        .filter_map(|v| v.get("VulnerabilityID")?.as_str().map(str::to_owned))
        .collect();
    ids.sort();
    ids.dedup();
    Ok(ids)
}
