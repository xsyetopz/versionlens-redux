use versionlens_parsers::DocumentInput;

use crate::{RegistryResponseInput, SessionConfig};

use super::{package_file_fixture, test_indicators};
use versionlens_parsers::Ecosystem::Docker;

#[test]
fn docker_argument_image_name_uses_not_supported_status() {
    let lenses = docker_code_lenses(package_file_fixture("Dockerfile-arg-image").as_str());

    assert_eq!(lenses, [("N not supported".to_owned(), "".to_owned())]);
}

#[test]
fn docker_argument_image_version_uses_not_supported_status() {
    let lenses = docker_code_lenses(package_file_fixture("Dockerfile-arg-version").as_str());

    assert_eq!(lenses, [("N not supported".to_owned(), "".to_owned())]);
}

fn docker_code_lenses(text: &str) -> Vec<(String, String)> {
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
        uri: "file:///Dockerfile".to_owned(),
        language_id: "dockerfile".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    };

    session.resolve_document_with_responses(input.clone(), &[]);
    session
        .analyze_document(input)
        .code_lenses
        .into_iter()
        .map(|lens| (lens.title, lens.command))
        .collect()
}

#[test]
fn docker_code_lenses_offer_same_suffix_update_choices() {
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
        uri: "file:///Dockerfile".to_owned(),
        language_id: "dockerfile".to_owned(),
        text: package_file_fixture("Dockerfile-node-20-bookworm"),
        workspace_root: None,
    };

    session.resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput {
            package: "node".to_owned(),
            ecosystem: Docker,
            body: r#"{"results":[{"name":"20.19.1-bookworm","tag_status":"active","digest":"sha256-20-bookworm"},{"name":"21.0.0-alpine","tag_status":"active","digest":"sha256-21-alpine"},{"name":"23.11.0-bookworm","tag_status":"active","digest":"sha256-23-bookworm"}]}"#
                .to_owned(),
        }],
    );

    let output = session.analyze_document(input);
    let arguments = output
        .code_lenses
        .iter()
        .filter_map(|lens| {
            if lens.command != "versionlens.suggestion.onUpdateDependency" {
                return None;
            }
            let command = lens.arguments.get(2)?;
            let version = lens.arguments.get(3)?;
            Some(vec![command.as_str(), version.as_str()])
        })
        .collect::<Vec<_>>();

    assert_eq!(arguments, [vec!["update", "23.11.0-bookworm"]]);
}

#[test]
fn docker_code_lenses_map_latest_update_choice_to_matching_tag_shape() {
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
        uri: "file:///Dockerfile".to_owned(),
        language_id: "dockerfile".to_owned(),
        text: package_file_fixture("Dockerfile-node-22-bookworm"),
        workspace_root: None,
    };

    session.resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput {
            package: "node".to_owned(),
            ecosystem: Docker,
            body: r#"{"results":[{"name":"latest","tag_status":"active","digest":"sha256-23"},{"name":"current-bookworm","tag_status":"active","digest":"sha256-23"},{"name":"current","tag_status":"active","digest":"sha256-23"},{"name":"bookworm","tag_status":"active","digest":"sha256-23"},{"name":"23.11.0-bookworm","tag_status":"active","digest":"sha256-23"},{"name":"23.11.0","tag_status":"active","digest":"sha256-23"},{"name":"23.11-bookworm","tag_status":"active","digest":"sha256-23"},{"name":"23.11","tag_status":"active","digest":"sha256-23"},{"name":"23-bookworm","tag_status":"active","digest":"sha256-23"},{"name":"23","tag_status":"active","digest":"sha256-23"},{"name":"22.4.3","tag_status":"active","digest":"sha256-22"},{"name":"22.4","tag_status":"active","digest":"sha256-22"},{"name":"22-bookworm","tag_status":"active","digest":"sha256-22"},{"name":"22","tag_status":"active","digest":"sha256-22"},{"name":"21.0.0","tag_status":"active","digest":"sha256-21"},{"name":"21.0","tag_status":"active","digest":"sha256-21"}]}"#
                .to_owned(),
        }],
    );

    let output = session.analyze_document(input);
    let arguments = output
        .code_lenses
        .iter()
        .filter_map(|lens| {
            if lens.command != "versionlens.suggestion.onUpdateDependency" {
                return None;
            }
            let command = lens.arguments.get(2)?;
            let version = lens.arguments.get(3)?;
            Some(vec![command.as_str(), version.as_str()])
        })
        .collect::<Vec<_>>();
    assert_eq!(arguments, [vec!["update", "23-bookworm"]]);
    assert_eq!(
        output
            .code_lenses
            .iter()
            .find(|lens| lens.command == "versionlens.suggestion.onChooseBuild")
            .map(|lens| lens
                .arguments
                .iter()
                .skip(1)
                .map(|value| value.as_str())
                .collect()),
        Some(vec![
            "node",
            "22-bookworm",
            "22",
            "22-bookworm",
            "22.4",
            "22.4.3"
        ])
    );
}

#[test]
fn docker_code_lenses_offer_update_choices_for_missing_numeric_tag() {
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
        uri: "file:///Dockerfile".to_owned(),
        language_id: "dockerfile".to_owned(),
        text: package_file_fixture("Dockerfile-node-21"),
        workspace_root: None,
    };

    session.resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput {
            package: "node".to_owned(),
            ecosystem: Docker,
            body: r#"{"results":[{"name":"latest","tag_status":"active","digest":"sha256-23"},{"name":"current-bookworm","tag_status":"active","digest":"sha256-23"},{"name":"current","tag_status":"active","digest":"sha256-23"},{"name":"bookworm","tag_status":"active","digest":"sha256-23"},{"name":"23.11.0-bookworm","tag_status":"active","digest":"sha256-23"},{"name":"23.11.0","tag_status":"active","digest":"sha256-23"},{"name":"23.11-bookworm","tag_status":"active","digest":"sha256-23"},{"name":"23.11","tag_status":"active","digest":"sha256-23"},{"name":"23-bookworm","tag_status":"active","digest":"sha256-23"},{"name":"23","tag_status":"active","digest":"sha256-23"},{"name":"22.4.3","tag_status":"active","digest":"sha256-22"},{"name":"22.4","tag_status":"active","digest":"sha256-22"},{"name":"22-bookworm","tag_status":"active","digest":"sha256-22"},{"name":"22","tag_status":"active","digest":"sha256-22"},{"name":"21.0.0","tag_status":"active","digest":"sha256-21"},{"name":"21.0","tag_status":"active","digest":"sha256-21"}]}"#
                .to_owned(),
        }],
    );

    let output = session.analyze_document(input);
    let titles = output
        .code_lenses
        .iter()
        .map(|lens| lens.title.as_str())
        .collect::<Vec<_>>();
    let arguments = output
        .code_lenses
        .iter()
        .skip(1)
        .map(|lens| {
            lens.arguments
                .iter()
                .skip(2)
                .map(|value| value.as_str())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    assert_eq!(titles, ["N no match", "U latest 23", "U major 22"]);
    assert_eq!(arguments, [vec!["update", "23"], vec!["updateMajor", "22"]]);
}

#[test]
fn docker_code_lenses_offer_latest_for_untagged_non_version_latest() {
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
        uri: "file:///Dockerfile".to_owned(),
        language_id: "dockerfile".to_owned(),
        text: package_file_fixture("Dockerfile-mssql-server"),
        workspace_root: None,
    };

    session.resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput {
            package: "mssql/server".to_owned(),
            ecosystem: Docker,
            body: r#"{"results":[{"name":"2022-RTM-CU2-ubuntu-20.04","tag_status":"active","digest":"sha256-a"},{"name":"2022-RTM-GDR1-ubuntu-20.04","tag_status":"active","digest":"sha256-b"},{"name":"2022-RTM-ubuntu-20.04","tag_status":"active","digest":"sha256-c"},{"name":"2022-latest","tag_status":"active","digest":"sha256-latest"},{"name":"2022-preview-ubuntu-22.04","tag_status":"active","digest":"sha256-d"},{"name":"latest","tag_status":"active","digest":"sha256-latest"},{"name":"latest-ubuntu","tag_status":"active","digest":"sha256-e"}]}"#
                .to_owned(),
        }],
    );

    let output = session.analyze_document(input);
    let titles = output
        .code_lenses
        .iter()
        .map(|lens| lens.title.as_str())
        .collect::<Vec<_>>();
    let arguments = output
        .code_lenses
        .iter()
        .skip(1)
        .map(|lens| {
            lens.arguments
                .iter()
                .skip(2)
                .map(|value| value.as_str())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    assert_eq!(titles, ["N no match", "U latest latest"]);
    assert_eq!(arguments, [vec!["update", "latest"]]);
}
