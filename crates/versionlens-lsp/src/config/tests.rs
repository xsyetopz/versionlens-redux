use serde_json::json;

use super::*;
use crate::registry;

#[test]
fn reads_partial_sections_with_defaults() {
    let settings = Settings::from_json(
        &json!({ "versionlens": { "prereleases": true, "timeoutMs": 2000, "registries": { "npm": "https://mirror/{enc}" } } }),
    );
    assert!(settings.prereleases && settings.enabled);
    assert_eq!(settings.timeout, Duration::from_secs(2));
    assert_eq!(settings.registries["npm"], "https://mirror/{enc}");
    assert_eq!(Settings::from_json(&json!(null)).concurrency, 32);
}

#[test]
fn reads_string_maps_and_drops_blank_values() {
    let settings = Settings::from_json(&json!({
        "authorization": { "https://npm.example.com/": "Bearer t", "https://x/": " " },
        "install": { "npm": "npm install", "cargo": 3 },
        "files": { "deps/*.txt": "requirements" },
    }));
    assert_eq!(settings.authorization.len(), 1);
    assert_eq!(settings.install.len(), 1);
    assert_eq!(settings.files["deps/*.txt"], "requirements");
}

#[test]
fn reads_indicators_lists_and_severity() {
    let settings = Settings::from_json(&json!({
        "indicators": { "latest": "ok", "update": 1, "build": "b" },
        "prereleaseTags": ["beta", " ", 2],
        "updateSeverity": "off",
        "fileTypes": ["npm"],
    }));
    assert_eq!(settings.indicators.latest, "ok");
    assert_eq!(settings.indicators.update, Indicators::default().update);
    assert_eq!(settings.indicators.build, "b");
    assert_eq!(settings.prerelease_tags, ["beta"]);
    assert_eq!(settings.update_severity, None);
    assert_eq!(settings.file_types, ["npm"]);
    let default = Settings::from_json(&json!({}));
    assert_eq!(
        default.update_severity,
        Some(DiagnosticSeverity::INFORMATION)
    );
}

#[test]
fn providers_override_one_registry_and_keep_global_for_others() {
    let settings = Settings::from_json(&json!({
        "cacheMinutes": 10,
        "providers": { "npm": { "cacheMinutes": 1, "strictSsl": false, "prereleaseTags": ["rc"] } },
    }));
    assert_eq!(settings.cache_for("npm"), Duration::from_secs(60));
    assert_eq!(settings.cache_for("cargo"), Duration::from_secs(600));
    assert!(!settings.strict_ssl_for("npm") && settings.strict_ssl_for("cargo"));
    assert_eq!(settings.prerelease_tags_for("npm"), ["rc"]);
    assert_eq!(settings.prerelease_tags_for("cargo"), [] as [String; 0]);
}

#[test]
fn reads_original_settings_under_their_new_names() {
    let settings = Settings::from_json(&json!({ "versionlens": {
        "caching": { "duration": 3 },
        "http": { "strictSSL": false },
        "suggestions": { "showOnStartup": false, "showPrereleasesOnStartup": true,
            "indicators": { "Latest": "🟢", "Match": "🟡", "Directory": "D", "Yanked": "Y" } },
        "enabledProviders": ["golang", "npm"],
        "golang": { "caching": { "duration": 0 }, "prereleaseTagFilter": ["beta"] },
        "npm": { "onSaveChanges": "npm install", "files": "**/{package.json,deps/*.json}" },
        "cargo": { "apiUrl": "https://crates.example/api" },
        "pub": { "apiUrl": "https://pub.example/api/packages/" },
        "maven": { "apiUrl": "" },
        "dotnet": { "nuget": { "sources": ["https://feed.example/v3/index.json"] } },
        "pypi": { "dependencyProperties": ["project", "tool.poetry.dependencies"] },
    }}));
    let poetry = ["project", "tool.poetry.dependencies"];
    assert_eq!(settings.properties["pyproject"], poetry);
    assert_eq!(settings.properties["pipfile"], poetry);
    assert!(!settings.properties.contains_key("requirements"));
    assert_eq!(
        settings.nuget_sources,
        ["https://feed.example/v3/index.json"]
    );
    assert_eq!(settings.cache, Duration::from_secs(180));
    assert!(!settings.strict_ssl && !settings.enabled && settings.prereleases);
    assert_eq!(settings.indicators.latest, "🟢");
    assert_eq!(
        (
            settings.indicators.directory.as_str(),
            settings.indicators.yanked.as_str()
        ),
        ("D", "Y")
    );
    assert_eq!(settings.file_types, ["go", "npm"]);
    assert_eq!(settings.cache_for("go"), Duration::ZERO);
    assert_eq!(settings.prerelease_tags_for("go"), ["beta"]);
    assert_eq!(settings.install["npm"], "npm install");
    assert_eq!(settings.files["**/package.json"], "npm");
    assert_eq!(settings.files["**/deps/*.json"], "npm");
    assert_eq!(settings.unread, Vec::<String>::new());
    assert_eq!(
        settings.registries["cargo"],
        "https://crates.example/api/{name}/versions"
    );
    assert_eq!(
        settings.registries["pub"],
        "https://pub.example/api/packages/{name}"
    );
    assert!(!settings.registries.contains_key("maven"));
}

#[test]
fn original_api_urls_become_registry_templates() {
    // Arrange: each provider's default `apiUrl` in upstream 1.3.0's package.json.
    let section = json!({ "versionlens": {
        "registries": { "rubygems": "https://gems.example/{name}" },
        "dub": { "apiUrl": "https://code.dlang.org/api/packages" },
        "composer": { "apiUrl": "https://repo.packagist.org/p2" },
        "maven": { "apiUrl": "https://repo.example/maven2/" },
        "golang": { "apiUrl": "https://proxy.golang.org/{base-module}/@v/list" },
        "ruby": { "apiUrl": "https://rubygems.org/api/v1/versions/{name}.json" },
        "pypi": { "apiUrl": "https://test.pypi.org/pypi/{name}/json" },
        "cargo": { "apiUrl": "https://crates.io/api/v1/crates" },
    }});

    // Act
    let settings = Settings::from_json(&section);

    // Assert: each matches the row's own URL, and a new `registries` key wins.
    let template = |id: &str| registry::get(id).unwrap().url;
    assert_eq!(settings.registries["dub"], template("dub"));
    assert_eq!(settings.registries["composer"], template("composer"));
    assert_eq!(
        settings.registries["maven"],
        "https://repo.example/maven2/{path}/maven-metadata.xml"
    );
    assert_eq!(settings.registries["go"], template("go"));
    assert_eq!(
        settings.registries["pypi"],
        "https://test.pypi.org/simple/{name}/"
    );
    assert_eq!(settings.registries["cargo"], template("cargo"));
    assert_eq!(
        settings.registries["rubygems"],
        "https://gems.example/{name}"
    );
    assert!(settings.unread.is_empty(), "{:?}", settings.unread);
}

#[test]
fn new_keys_win_over_original_ones() {
    let settings = Settings::from_json(&json!({
        "cacheMinutes": 5,
        "caching": { "duration": 3 },
        "providers": { "go": { "cacheMinutes": 2 } },
        "golang": { "caching": { "duration": 9 } },
    }));
    assert_eq!(settings.cache, Duration::from_secs(300));
    assert_eq!(settings.cache_for("go"), Duration::from_secs(120));
}

#[test]
fn takes_github_token_from_setting_then_github_token_then_gh_token() {
    let env = |set: &'static [(&'static str, &'static str)]| {
        move |k: &str| {
            set.iter()
                .find(|(n, _)| *n == k)
                .map(|(_, v)| (*v).to_owned())
        }
    };
    let both = env(&[("GITHUB_TOKEN", "a"), ("GH_TOKEN", "b")]);
    assert_eq!(
        github_token(Some("s".to_owned()), both).as_deref(),
        Some("s")
    );
    assert_eq!(github_token(None, both).as_deref(), Some("a"));
    assert_eq!(
        github_token(None, env(&[("GITHUB_TOKEN", ""), ("GH_TOKEN", "b")])).as_deref(),
        Some("b")
    );
    assert_eq!(github_token(None, env(&[])), None);
}
