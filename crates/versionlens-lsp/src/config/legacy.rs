//! The original `VersionLens` 1.x settings, read as fallbacks so a `settings.json` written for it
//! keeps working. Each maps onto the setting that replaced it, and a key of this rewrite wins.

use serde_json::Value;

use super::{Settings, list, minutes};
use crate::catalog;

/// Each original provider name, the registry id it became, and the file type of its manifests.
const PROVIDERS: &[(&str, &str, &str)] = &[
    ("cargo", "cargo", "cargo"),
    ("composer", "composer", "composer"),
    ("deno", "jsr", "deno"),
    ("docker", "docker", "dockerfile"),
    ("dotnet", "nuget", "nuget"),
    ("dub", "dub", "dub"),
    ("golang", "go", "go"),
    ("maven", "maven", "maven"),
    ("npm", "npm", "npm"),
    ("pnpm", "npm", "pnpm"),
    ("pub", "pub", "pubspec"),
    ("pypi", "pypi", "requirements"),
    ("ruby", "rubygems", "gemfile"),
];

/// Original provider keys this rewrite has no counterpart for: its registry URLs are templates
/// in `registries`, and its extraction rules live in the catalog.
const UNREAD: &[&str] = &["apiUrl", "dependencyProperties", "nuget"];

/// Original indicator names and the glyph each became; the others name states this rewrite
/// does not draw.
const INDICATORS: &[(&str, &str)] = &[
    ("Latest", "latest"),
    ("Updateable", "update"),
    ("Error", "failed"),
    ("UpdateableVulnerable", "vulnerable"),
    ("Match", "satisfies"),
    ("NoMatch", "noMatch"),
];

/// Fills every setting the section leaves unset from its original key, when that is present.
pub(super) fn apply(section: &Value, settings: &mut Settings) {
    let unset = |key: &str| section.get(key).is_none();
    if unset("cacheMinutes")
        && let Some(cache) = section.pointer("/caching/duration").and_then(minutes)
    {
        settings.cache = cache;
    }
    if unset("strictSsl")
        && let Some(strict) = section.pointer("/http/strictSSL").and_then(Value::as_bool)
    {
        settings.strict_ssl = strict;
    }
    for (old, new, field) in [
        ("showOnStartup", "enabled", &mut settings.enabled),
        (
            "showPrereleasesOnStartup",
            "prereleases",
            &mut settings.prereleases,
        ),
        (
            "showVulnerabilities",
            "vulnerabilities",
            &mut settings.vulnerabilities,
        ),
        ("showSuggestionsStats", "stats", &mut settings.stats),
    ] {
        if unset(new)
            && let Some(flag) = section
                .pointer(&format!("/suggestions/{old}"))
                .and_then(Value::as_bool)
        {
            *field = flag;
        }
    }
    if unset("indicators")
        && let Some(old) = section.pointer("/suggestions/indicators")
    {
        let mapped = INDICATORS
            .iter()
            .filter_map(|(from, to)| Some((*to, old.get(from)?.as_str()?.trim())))
            .map(|(to, glyph)| (to.to_owned(), Value::from(glyph)));
        settings.indicators = super::Indicators::from_json(Some(&mapped.collect()));
    }
    if unset("fileTypes") && section.get("enabledProviders").is_some() {
        settings.file_types = list(section, "enabledProviders")
            .iter()
            .filter_map(|name| PROVIDERS.iter().find(|p| p.0 == name))
            .map(|p| p.1.to_owned())
            .collect();
    }
    for &(name, registry, file_type) in PROVIDERS {
        if let Some(old) = section.get(name).filter(|v| v.is_object()) {
            provider(old, name, registry, file_type, settings);
        }
    }
}

fn provider(old: &Value, name: &str, registry: &str, file_type: &str, settings: &mut Settings) {
    let entry = settings.providers.entry(registry.to_owned()).or_default();
    if entry.cache.is_none() {
        entry.cache = old.pointer("/caching/duration").and_then(minutes);
    }
    if entry.strict_ssl.is_none() {
        entry.strict_ssl = old.pointer("/http/strictSSL").and_then(Value::as_bool);
    }
    if entry.prerelease_tags.is_none() && old.get("prereleaseTagFilter").is_some() {
        entry.prerelease_tags = Some(list(old, "prereleaseTagFilter"));
    }
    if let Some(command) = old
        .get("onSaveChanges")
        .and_then(Value::as_str)
        .filter(|c| !c.trim().is_empty())
    {
        settings
            .install
            .entry(file_type.to_owned())
            .or_insert_with(|| command.to_owned());
    }
    if let Some(glob) = old.get("files").and_then(Value::as_str) {
        for glob in braces(glob) {
            // A glob that names a catalogued file keeps that file type; others read as the
            // provider's main manifest.
            let id = catalog::file_type(&glob)
                .filter(|f| f.registry == registry)
                .map_or(file_type, |f| f.id);
            settings.files.entry(glob).or_insert_with(|| id.to_owned());
        }
    }
    for key in UNREAD {
        if old.get(key).is_some() {
            settings.unread.push(format!("versionlens.{name}.{key}"));
        }
    }
}

/// Expands the one `{a,b}` group the original's globs used into a glob per alternative.
fn braces(glob: &str) -> Vec<String> {
    match (glob.find('{'), glob.find('}')) {
        (Some(open), Some(close)) if open < close => glob[open + 1..close]
            .split(',')
            .map(|alt| format!("{}{alt}{}", &glob[..open], &glob[close + 1..]))
            .collect(),
        _ => vec![glob.to_owned()],
    }
}
