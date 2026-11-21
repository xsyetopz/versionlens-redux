//! User settings, read from the `versionlens` section that every editor sends.

use std::collections::HashMap;
use std::time::Duration;

use serde_json::Value;

/// Server settings; every field has a default, so a missing or partial section still works.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Settings {
    pub(crate) enabled: bool,
    pub(crate) code_lens: bool,
    pub(crate) inlay_hints: bool,
    pub(crate) prereleases: bool,
    pub(crate) vulnerabilities: bool,
    pub(crate) timeout: Duration,
    pub(crate) concurrency: usize,
    pub(crate) cache: Duration,
    pub(crate) strict_ssl: bool,
    pub(crate) github_token: Option<String>,
    /// Per-registry URL template overrides, keyed by registry id, for mirrors and private registries.
    pub(crate) registries: HashMap<String, String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            enabled: true,
            code_lens: true,
            inlay_hints: true,
            prereleases: false,
            vulnerabilities: true,
            timeout: Duration::from_secs(10),
            concurrency: 32,
            cache: Duration::from_secs(3600),
            strict_ssl: true,
            github_token: None,
            registries: HashMap::new(),
        }
    }
}

impl Settings {
    /// Reads settings from either the `versionlens` section or an object holding it.
    pub(crate) fn from_json(value: &Value) -> Self {
        let section = value.get("versionlens").unwrap_or(value);
        let base = Self::default();
        let flag =
            |key: &str, default: bool| section.get(key).and_then(Value::as_bool).unwrap_or(default);
        let number =
            |key: &str, default: u64| section.get(key).and_then(Value::as_u64).unwrap_or(default);
        let token = section
            .get("githubToken")
            .and_then(Value::as_str)
            .filter(|t| !t.is_empty())
            .map(str::to_owned);
        Self {
            enabled: flag("enabled", base.enabled),
            code_lens: flag("codeLens", base.code_lens),
            inlay_hints: flag("inlayHints", base.inlay_hints),
            prereleases: flag("prereleases", base.prereleases),
            vulnerabilities: flag("vulnerabilities", base.vulnerabilities),
            timeout: Duration::from_millis(number("timeoutMs", 10_000).max(500)),
            concurrency: usize::try_from(number("concurrency", 32).clamp(1, 256)).unwrap_or(32),
            cache: Duration::from_secs(number("cacheMinutes", 60) * 60),
            strict_ssl: flag("strictSsl", base.strict_ssl),
            github_token: token.or_else(|| {
                ["GITHUB_TOKEN", "GH_TOKEN"]
                    .iter()
                    .find_map(|k| std::env::var(k).ok())
            }),
            registries: section
                .get("registries")
                .and_then(Value::as_object)
                .map_or_default(|map| {
                    map.iter()
                        .filter_map(|(k, v)| Some((k.clone(), v.as_str()?.to_owned())))
                        .collect()
                }),
        }
    }
}

#[cfg(test)]
mod tests;
