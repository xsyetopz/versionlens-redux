use std::env::temp_dir;
use std::fs::create_dir_all;
use std::fs::remove_dir_all;
use std::fs::write;
use std::path::{Path, PathBuf};
use std::process::id;
use std::time::UNIX_EPOCH;

use versionlens_model::DocumentInput;

use crate::{RegistryResponseInput, SessionConfig};

use super::{package_file_fixture, test_indicators};
use versionlens_model::Ecosystem::Npm;

#[test]
fn project_version_code_lenses_offer_stable_bumps() {
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: crate::default(),
        suggestion_indicators: test_indicators(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    });
    let input = DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture("package-version-1.2.3.json"),
        workspace_root: None,
    };

    let output = session.analyze_document(input);
    let titles = output
        .code_lenses
        .iter()
        .map(|lens| lens.title.as_str())
        .collect::<Vec<_>>();
    let commands = output
        .code_lenses
        .iter()
        .filter_map(|lens| lens.arguments.get(2).map(|value| value.as_str()))
        .collect::<Vec<_>>();

    assert_eq!(titles, ["U major 2.0.0", "U minor 1.3.0", "U patch 1.2.4"]);
    assert_eq!(commands, ["updateMajor", "updateMinor", "updatePatch"]);
}

#[test]
fn project_version_code_lenses_offer_prerelease_bumps() {
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: crate::default(),
        suggestion_indicators: test_indicators(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    });
    let input = DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture("package-version-1.2.3-beta.4.json"),
        workspace_root: None,
    };

    let output = session.analyze_document(input);
    let titles = output
        .code_lenses
        .iter()
        .map(|lens| lens.title.as_str())
        .collect::<Vec<_>>();
    let commands = output
        .code_lenses
        .iter()
        .filter_map(|lens| lens.arguments.get(2).map(|value| value.as_str()))
        .collect::<Vec<_>>();

    assert_eq!(titles, ["U release 1.2.3", "U prerelease 1.2.3-beta.5"]);
    assert_eq!(commands, ["updateRelease", "updatePrerelease"]);
}

#[test]
fn build_code_lens_chooses_available_build_versions() {
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: crate::default(),
        suggestion_indicators: test_indicators(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    });
    let input = DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture("package-left-pad-1.0.0-build.1.json"),
        workspace_root: None,
    };

    session.resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Npm,
            body: r#"{
              "dist-tags": { "latest": "1.0.0+build.2" },
              "versions": {
                "1.0.0": {},
                "1.0.0+build.1": {},
                "1.0.0+build.2": {},
                "1.0.0+build.3": {},
                "1.1.0": {}
              }
            }"#
            .to_owned(),
        }],
    );

    let output = session.analyze_document(input);
    let titles = output
        .code_lenses
        .iter()
        .map(|lens| lens.title.as_str())
        .collect::<Vec<_>>();
    let commands = output
        .code_lenses
        .iter()
        .map(|lens| lens.command.as_str())
        .collect::<Vec<_>>();

    assert_eq!(titles, ["L latest 1.0.0+build.1", "B change build"]);
    assert_eq!(commands, ["", "versionlens.suggestion.onChooseBuild"]);
    assert_eq!(
        &output.code_lenses[1].arguments[1..],
        [
            "left-pad",
            "1.0.0+build.1",
            "1.0.0",
            "1.0.0+build.1",
            "1.0.0+build.2",
            "1.0.0+build.3"
        ]
    );
}

#[test]
fn build_code_lens_keeps_latest_status_when_current_has_build_versions() {
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: crate::default(),
        suggestion_indicators: test_indicators(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    });
    let input = DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture("package-left-pad-3.0.0.json"),
        workspace_root: None,
    };

    session.resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Npm,
            body: r#"{
              "dist-tags": { "latest": "3.0.0" },
              "versions": {
                "1.0.0": {},
                "2.0.0": {},
                "2.1.0": {},
                "3.0.0": {},
                "3.0.0+b1": {},
                "3.0.0+b2": {}
              }
            }"#
            .to_owned(),
        }],
    );

    let output = session.analyze_document(input);
    let titles = output
        .code_lenses
        .iter()
        .map(|lens| lens.title.as_str())
        .collect::<Vec<_>>();
    let commands = output
        .code_lenses
        .iter()
        .map(|lens| lens.command.as_str())
        .collect::<Vec<_>>();

    assert_eq!(titles, ["L latest 3.0.0", "B change build"]);
    assert_eq!(commands, ["", "versionlens.suggestion.onChooseBuild"]);
    assert_eq!(
        &output.code_lenses[1].arguments[1..],
        ["left-pad", "3.0.0", "3.0.0", "3.0.0+b1", "3.0.0+b2"]
    );
}

#[test]
fn build_code_lens_keeps_latest_status_when_current_build_differs_from_latest_build() {
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: crate::default(),
        suggestion_indicators: test_indicators(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    });
    let input = DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture("package-left-pad-3.0.0-b1.json"),
        workspace_root: None,
    };

    session.resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Npm,
            body: r#"{
              "dist-tags": { "latest": "3.0.0+b2" },
              "versions": {
                "1.0.0": {},
                "2.0.0": {},
                "2.1.0": {},
                "3.0.0": {},
                "3.0.0+b1": {},
                "3.0.0+b2": {}
              }
            }"#
            .to_owned(),
        }],
    );

    let output = session.analyze_document(input);
    let titles = output
        .code_lenses
        .iter()
        .map(|lens| lens.title.as_str())
        .collect::<Vec<_>>();
    let commands = output
        .code_lenses
        .iter()
        .map(|lens| lens.command.as_str())
        .collect::<Vec<_>>();

    assert_eq!(titles, ["L latest 3.0.0+b1", "B change build"]);
    assert_eq!(commands, ["", "versionlens.suggestion.onChooseBuild"]);
    assert_eq!(
        &output.code_lenses[1].arguments[1..],
        ["left-pad", "3.0.0+b1", "3.0.0", "3.0.0+b1", "3.0.0+b2"]
    );
}

#[test]
fn build_code_lens_uses_latest_build_when_variant_list_is_missing() {
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: crate::default(),
        suggestion_indicators: test_indicators(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    });
    let input = DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture("package-left-pad-1.0.0-build.1.json"),
        workspace_root: None,
    };

    session.resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Npm,
            body: r#"{"dist-tags":{"latest":"1.0.0+build.2"}}"#.to_owned(),
        }],
    );

    let output = session.analyze_document(input);

    assert_eq!(output.code_lenses[0].title, "B change build");
    assert_eq!(
        output.code_lenses[0].command,
        "versionlens.suggestion.onChooseBuild"
    );
    assert_eq!(
        &output.code_lenses[0].arguments[1..],
        ["left-pad", "1.0.0+build.1", "1.0.0+build.2"]
    );
}

#[test]
fn directory_code_lens_opens_local_dependency_path() {
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: crate::default(),
        suggestion_indicators: test_indicators(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    });
    let root = local_test_root("directory-codelens");
    let app = root.join("app");
    let local = root.join("local");
    create_dir_all(&app).unwrap();
    create_dir_all(&local).unwrap();
    let input = DocumentInput {
        uri: file_uri(&app.join("package.json")),
        language_id: "json".to_owned(),
        text: package_file_fixture("package-local-file-dependency.json"),
        workspace_root: None,
    };

    session.resolve_document(input.clone());
    let output = session.analyze_document(input);

    let local_path = local.to_string_lossy();
    assert_eq!(output.code_lenses[0].title, "D file://../local");
    assert_eq!(
        output.code_lenses[0].command,
        "versionlens.suggestion.onFileLink"
    );
    assert_eq!(output.code_lenses[0].arguments, [local_path.as_ref()]);
    remove_dir_all(root).unwrap();
}

#[test]
fn npm_link_code_lens_opens_package_json_target_path() {
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: crate::default(),
        suggestion_indicators: test_indicators(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    });
    let root = local_test_root("npm-link-codelens");
    let app = root.join("app");
    let local = root.join("local");
    create_dir_all(&app).unwrap();
    create_dir_all(&local).unwrap();
    write(
        local.join("package.json"),
        package_file_fixture("empty-package.json"),
    )
    .unwrap();
    let input = DocumentInput {
        uri: file_uri(&app.join("package.json")),
        language_id: "json".to_owned(),
        text: package_file_fixture("package-local-link-dependency.json"),
        workspace_root: None,
    };

    session.resolve_document(input.clone());
    let output = session.analyze_document(input);

    let target_path = local.join("package.json");
    let target_path = target_path.to_string_lossy();
    assert_eq!(
        output.code_lenses[0].title,
        "D file://../local/package.json"
    );
    assert_eq!(
        output.code_lenses[0].command,
        "versionlens.suggestion.onFileLink"
    );
    assert_eq!(output.code_lenses[0].arguments, [target_path.as_ref()]);
    remove_dir_all(root).unwrap();
}

#[test]
fn missing_directory_code_lens_is_disabled_not_found_status() {
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: crate::default(),
        suggestion_indicators: test_indicators(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    });
    let input = DocumentInput {
        uri: "file:///repo/app/package.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture("package-local-file-dependency.json"),
        workspace_root: None,
    };

    session.resolve_document(input.clone());
    let output = session.analyze_document(input);

    assert_eq!(output.code_lenses[0].title, "E not found ../local");
    assert_eq!(output.code_lenses[0].command, "");
    assert!(output.code_lenses[0].arguments.is_empty());
}

fn local_test_root(name: &str) -> PathBuf {
    let root = temp_dir().join(format!(
        "versionlens-{name}-{}-{}",
        id(),
        crate::system_time_now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    create_dir_all(&root).unwrap();
    root
}

fn file_uri(path: &Path) -> String {
    format!("file://{}", path.to_string_lossy())
}
