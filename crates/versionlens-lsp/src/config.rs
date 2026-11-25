//! User settings, read from the `versionlens` section that every editor sends.

use std::collections::HashMap;
use std::time::Duration;

use lsp_types::DiagnosticSeverity;
use serde_json::Value;

/// The glyphs that lenses and inlay hints start with, so users can swap them for emoji or ASCII.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Indicators {
    pub(crate) latest: String,
    pub(crate) update: String,
    pub(crate) checking: String,
    pub(crate) failed: String,
    pub(crate) unreadable: String,
    pub(crate) vulnerable: String,
    pub(crate) newer: String,
    /// The requirement allows a listed version, but not the latest.
    pub(crate) satisfies: String,
    /// The registry lists no version the requirement allows.
    pub(crate) no_match: String,
    /// Leads a local dependency's path.
    pub(crate) directory: String,
    /// Leads the lens that switches between builds of the resolved version.
    pub(crate) build: String,
    /// The requirement resolves only to a yanked version.
    pub(crate) yanked: String,
    /// The registry marks the resolved version deprecated.
    pub(crate) deprecated: String,
}

/// The original 1.x glyphs, which GUI editors show as is; terminal editors such as Neovim send
/// their own text glyphs instead.
impl Default for Indicators {
    fn default() -> Self {
        Self {
            latest: "🟢".into(),
            update: "↑ ".into(),
            checking: "⋯".into(),
            failed: "🔴".into(),
            unreadable: "?".into(),
            vulnerable: "⚠️".into(),
            newer: "→".into(),
            satisfies: "🟡".into(),
            no_match: "⚪".into(),
            directory: "📁".into(),
            build: "≌".into(),
            yanked: "🔴".into(),
            deprecated: "🟡".into(),
        }
    }
}

impl Indicators {
    fn from_json(value: Option<&Value>) -> Self {
        let mut out = Self::default();
        let Some(map) = value.and_then(Value::as_object) else {
            return out;
        };
        for (key, glyph) in [
            ("latest", &mut out.latest),
            ("update", &mut out.update),
            ("checking", &mut out.checking),
            ("failed", &mut out.failed),
            ("unreadable", &mut out.unreadable),
            ("vulnerable", &mut out.vulnerable),
            ("newer", &mut out.newer),
            ("satisfies", &mut out.satisfies),
            ("noMatch", &mut out.no_match),
            ("directory", &mut out.directory),
            ("build", &mut out.build),
            ("yanked", &mut out.yanked),
            ("deprecated", &mut out.deprecated),
        ] {
            if let Some(text) = map.get(key).and_then(Value::as_str) {
                text.clone_into(glyph);
            }
        }
        out
    }
}

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
    /// `Authorization` header values keyed by URL prefix, for private registries.
    pub(crate) authorization: HashMap<String, String>,
    /// Shell commands keyed by file type or registry id, run in the manifest's directory when a
    /// save changes its dependencies and by `versionlens.install`.
    pub(crate) install: HashMap<String, String>,
    /// Extra file globs keyed to the file type id that reads them.
    pub(crate) files: HashMap<String, String>,
    pub(crate) indicators: Indicators,
    /// When prereleases show, only those whose prerelease part holds one of these tags; empty
    /// keeps every prerelease.
    pub(crate) prerelease_tags: Vec<String>,
    /// The severity of the diagnostic each outdated dependency gets; `None` gives none.
    pub(crate) update_severity: Option<DiagnosticSeverity>,
    /// Whether a summary lens heads each manifest.
    pub(crate) stats: bool,
    /// The file type and registry ids to read; empty reads every one.
    pub(crate) file_types: Vec<String>,
    /// Globs of paths a workspace update skips.
    pub(crate) workspace_exclude: Vec<String>,
    /// Overrides keyed by registry id, as the original's per-provider settings were.
    pub(crate) providers: HashMap<String, Provider>,
    /// Original 1.x keys found in the section that have no counterpart, named for the log.
    pub(crate) unread: Vec<String>,
}

/// One registry's overrides of the global lookup settings; `None` keeps the global value.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Provider {
    pub(crate) cache: Option<Duration>,
    pub(crate) strict_ssl: Option<bool>,
    pub(crate) prerelease_tags: Option<Vec<String>>,
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
            authorization: HashMap::new(),
            install: HashMap::new(),
            files: HashMap::new(),
            indicators: Indicators::default(),
            prerelease_tags: Vec::new(),
            update_severity: Some(DiagnosticSeverity::INFORMATION),
            stats: false,
            file_types: Vec::new(),
            workspace_exclude: Vec::new(),
            providers: HashMap::new(),
            unread: Vec::new(),
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
        let mut settings = Self {
            enabled: flag("enabled", base.enabled),
            code_lens: flag("codeLens", base.code_lens),
            inlay_hints: flag("inlayHints", base.inlay_hints),
            prereleases: flag("prereleases", base.prereleases),
            vulnerabilities: flag("vulnerabilities", base.vulnerabilities),
            timeout: Duration::from_millis(number("timeoutMs", 10_000).max(500)),
            concurrency: usize::try_from(number("concurrency", 32).clamp(1, 256)).unwrap_or(32),
            cache: Duration::from_secs(number("cacheMinutes", 60) * 60),
            strict_ssl: flag("strictSsl", base.strict_ssl),
            github_token: github_token(token, |k| std::env::var(k).ok()),
            registries: strings(section, "registries"),
            authorization: strings(section, "authorization"),
            install: strings(section, "install"),
            files: strings(section, "files"),
            indicators: Indicators::from_json(section.get("indicators")),
            prerelease_tags: list(section, "prereleaseTags"),
            update_severity: match section.get("updateSeverity").and_then(Value::as_str) {
                Some("off") => None,
                Some("hint") => Some(DiagnosticSeverity::HINT),
                Some("warning") => Some(DiagnosticSeverity::WARNING),
                Some("error") => Some(DiagnosticSeverity::ERROR),
                _ => base.update_severity,
            },
            stats: flag("stats", base.stats),
            file_types: list(section, "fileTypes"),
            workspace_exclude: list(section, "workspaceExclude"),
            providers: providers(section),
            unread: Vec::new(),
        };
        legacy::apply(section, &mut settings);
        settings
    }

    /// How long a registry's answers stay fresh.
    pub(crate) fn cache_for(&self, registry: &str) -> Duration {
        self.providers
            .get(registry)
            .and_then(|p| p.cache)
            .unwrap_or(self.cache)
    }

    /// Whether a registry's certificates are verified.
    pub(crate) fn strict_ssl_for(&self, registry: &str) -> bool {
        self.providers
            .get(registry)
            .and_then(|p| p.strict_ssl)
            .unwrap_or(self.strict_ssl)
    }

    /// The prerelease tags a registry's prereleases must hold; empty keeps every one.
    pub(crate) fn prerelease_tags_for(&self, registry: &str) -> &[String] {
        self.providers
            .get(registry)
            .and_then(|p| p.prerelease_tags.as_deref())
            .unwrap_or(&self.prerelease_tags)
    }
}

/// A number of minutes as a duration; negative and non-numeric values are skipped.
fn minutes(value: &Value) -> Option<Duration> {
    value
        .as_f64()
        .filter(|m| m.is_finite() && *m >= 0.0)
        .map(|m| Duration::from_secs_f64(m * 60.0))
}

/// The `providers` setting: `cacheMinutes`, `strictSsl`, and `prereleaseTags` per registry id.
fn providers(section: &Value) -> HashMap<String, Provider> {
    section
        .get("providers")
        .and_then(Value::as_object)
        .map_or_default(|map| {
            map.iter()
                .map(|(id, entry)| {
                    let provider = Provider {
                        cache: entry.get("cacheMinutes").and_then(minutes),
                        strict_ssl: entry.get("strictSsl").and_then(Value::as_bool),
                        prerelease_tags: entry
                            .get("prereleaseTags")
                            .is_some()
                            .then(|| list(entry, "prereleaseTags")),
                    };
                    (id.clone(), provider)
                })
                .collect()
        })
}

/// A string-to-string object setting; other value types are skipped.
fn strings(section: &Value, key: &str) -> HashMap<String, String> {
    section
        .get(key)
        .and_then(Value::as_object)
        .map_or_default(|map| {
            map.iter()
                .filter_map(|(k, v)| Some((k.clone(), v.as_str()?.to_owned())))
                .filter(|(_, v)| !v.trim().is_empty())
                .collect()
        })
}

/// A string-list setting without blank entries; other value types are skipped.
fn list(section: &Value, key: &str) -> Vec<String> {
    section
        .get(key)
        .and_then(Value::as_array)
        .map_or_default(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_owned)
                .collect()
        })
}

mod legacy;

/// The GitHub token: the `githubToken` setting, then `GITHUB_TOKEN`, then `GH_TOKEN`, as gh itself
/// orders the two variables. `gh auth token` follows when all three are unset (`http::gh`).
fn github_token(setting: Option<String>, var: impl Fn(&str) -> Option<String>) -> Option<String> {
    setting.or_else(|| {
        ["GITHUB_TOKEN", "GH_TOKEN"]
            .iter()
            .find_map(|k| var(k).filter(|t| !t.is_empty()))
    })
}

#[cfg(test)]
mod tests;
