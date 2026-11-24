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
        "indicators": { "latest": "🟢", "update": 1 },
        "prereleaseTags": ["beta", " ", 2],
        "updateSeverity": "off",
        "fileTypes": ["npm"],
    }));
    assert_eq!(settings.indicators.latest, "🟢");
    assert_eq!(settings.indicators.update, "↑");
    assert_eq!(settings.prerelease_tags, ["beta"]);
    assert_eq!(settings.update_severity, None);
    assert_eq!(settings.file_types, ["npm"]);
    let default = Settings::from_json(&json!({}));
    assert_eq!(
        default.update_severity,
        Some(DiagnosticSeverity::INFORMATION)
    );
}
