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

/// Original indicator names and the glyph each became; `SatisfiesLatest` shares `latest`.
const INDICATORS: &[(&str, &str)] = &[
    ("Latest", "latest"),
    ("Updateable", "update"),
    ("Error", "failed"),
    ("UpdateableVulnerable", "vulnerable"),
    ("Match", "satisfies"),
    ("NoMatch", "noMatch"),
    ("Directory", "directory"),
    ("Build", "build"),
    ("Yanked", "yanked"),
    ("Deprecated", "deprecated"),
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
    if let Some(url) = old
        .get("apiUrl")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|u| !u.is_empty())
    {
        match template(name, url) {
            Some(template) => {
                settings
                    .registries
                    .entry(registry.to_owned())
                    .or_insert(template);
            }
            None => settings.unread.push(format!("versionlens.{name}.apiUrl")),
        }
    }
    // Upstream 1.3.0's nugetOptions.ts asks these service indexes in place of the sources
    // `dotnet nuget list source` prints.
    if let Some(nuget) = old.get("nuget") {
        settings.nuget_sources = list(nuget, "sources");
    }
    // Upstream 1.3.0 read a manifest only at these paths; its `pypi` provider read pyproject.toml
    // and Pipfile with them, and requirements.txt line by line.
    if old.get("dependencyProperties").is_some() {
        let paths = list(old, "dependencyProperties");
        let types: &[&str] = match name {
            "pypi" => &["pyproject", "pipfile"],
            _ => std::slice::from_ref(&file_type),
        };
        for id in types {
            settings
                .properties
                .entry((*id).to_owned())
                .or_insert_with(|| paths.clone());
        }
    }
}

/// The `registries` template an original `apiUrl` stands for, built the way each provider of
/// upstream 1.3.0 built its request URL from it: `{apiUrl}/{name}` in pubSuggestionProvider.ts,
/// `{apiUrl}/{name}/info` in dubJsonClient.ts, `{apiUrl}/{name}.json` in packagistClient.ts,
/// `{apiUrl}/{path}/maven-metadata.xml` in mavenHttpClient.ts, and a template with `{base-module}`
/// or `{name}` in goHttpClient.ts and rubyHttpClient.ts. A Warehouse index, as pypi.org and
/// test.pypi.org run, serves its `{base}/pypi/{name}/json` API beside the `{base}/simple/{name}/`
/// index the `pypi` row reads (docs.pypi.org/api/). The default `crates.io` web API maps to the
/// `cargo` row's own sparse index, and another host to `{apiUrl}/{name}/versions` as
/// cratesClient.ts built it, whose answer the `cargo` reader also reads.
fn template(name: &str, url: &str) -> Option<String> {
    let base = url.trim_end_matches('/');
    match name {
        "pypi" => base
            .strip_suffix("/pypi/{name}/json")
            .map(|host| format!("{host}/simple/{{name}}/")),
        "cargo" if base == "https://crates.io/api/v1/crates" => {
            crate::registry::get("cargo").map(|r| r.url.to_owned())
        }
        "cargo" => Some(format!("{base}/{{name}}/versions")),
        "pub" => Some(format!("{base}/{{name}}")),
        "dub" => Some(format!("{base}/{{name}}/info")),
        "composer" => Some(format!("{base}/{{name}}.json")),
        "maven" => Some(format!("{base}/{{path}}/maven-metadata.xml")),
        "golang" => Some(url.replace("{base-module}", "{go}")),
        "ruby" => Some(url.to_owned()),
        _ => None,
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
