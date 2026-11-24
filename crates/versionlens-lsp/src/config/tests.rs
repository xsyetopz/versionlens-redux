use serde_json::json;

use super::*;

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
        "indicators": { "latest": "ok", "update": 1 },
        "prereleaseTags": ["beta", " ", 2],
        "updateSeverity": "off",
        "fileTypes": ["npm"],
    }));
    assert_eq!(settings.indicators.latest, "ok");
    assert_eq!(settings.indicators.update, Indicators::default().update);
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
fn providers_override_one_registry_and_keep_the_global_for_others() {
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
fn reads_the_original_settings_under_their_new_names() {
    let settings = Settings::from_json(&json!({ "versionlens": {
        "caching": { "duration": 3 },
        "http": { "strictSSL": false },
        "suggestions": { "showOnStartup": false, "showPrereleasesOnStartup": true,
            "indicators": { "Latest": "🟢", "Match": "🟡" } },
        "enabledProviders": ["golang", "npm"],
        "golang": { "caching": { "duration": 0 }, "prereleaseTagFilter": ["beta"] },
        "npm": { "onSaveChanges": "npm install", "files": "**/{package.json,deps/*.json}" },
        "cargo": { "apiUrl": "https://crates.example/api" },
    }}));
    assert_eq!(settings.cache, Duration::from_secs(180));
    assert!(!settings.strict_ssl && !settings.enabled && settings.prereleases);
    assert_eq!(settings.indicators.latest, "🟢");
    assert_eq!(settings.file_types, ["go", "npm"]);
    assert_eq!(settings.cache_for("go"), Duration::ZERO);
    assert_eq!(settings.prerelease_tags_for("go"), ["beta"]);
    assert_eq!(settings.install["npm"], "npm install");
    assert_eq!(settings.files["**/package.json"], "npm");
    assert_eq!(settings.files["**/deps/*.json"], "npm");
    assert_eq!(settings.unread, ["versionlens.cargo.apiUrl"]);
}

#[test]
fn new_keys_win_over_the_original_ones() {
    let settings = Settings::from_json(&json!({
        "cacheMinutes": 5,
        "caching": { "duration": 3 },
        "providers": { "go": { "cacheMinutes": 2 } },
        "golang": { "caching": { "duration": 9 } },
    }));
    assert_eq!(settings.cache, Duration::from_secs(300));
    assert_eq!(settings.cache_for("go"), Duration::from_secs(120));
}
