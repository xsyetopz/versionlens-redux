use versionlens_http::HttpConfig;
use versionlens_parsers::{DocumentInput, Ecosystem};

use crate::{
    PrereleaseTagConfig, ProviderSettings, RegistryResponseInput, SessionConfig,
    SuggestionIndicators, VersionLensSession,
};

#[test]
fn show_prereleases_allows_prerelease_updates() {
    let session = VersionLensSession::new(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: Vec::new(),
        providers: ProviderSettings::default(),
        suggestion_indicators: SuggestionIndicators::standard(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: true,
        http: HttpConfig::standard(),
    });

    let input = DocumentInput {
            uri: "file:///Directory.Packages.props".to_owned(),
            language_id: "xml".to_owned(),
            text: r#"<Project><ItemGroup><PackageVersion Include="Newtonsoft.Json" Version="13.0.1" /></ItemGroup></Project>"#.to_owned(),
            workspace_root: None,
        };
    let output = session.resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput {
            package: "Newtonsoft.Json".to_owned(),
            ecosystem: Ecosystem::Dotnet,
            body: r#"{"versions":["13.0.3","14.0.0-beta.1"]}"#.to_owned(),
        }],
    );
    let analysis = session.analyze_document(input);
    let update_arguments = analysis
        .code_lenses
        .iter()
        .filter(|lens| lens.title == "↑  beta 14.0.0-beta.1")
        .map(|lens| {
            lens.arguments
                .iter()
                .skip(2)
                .map(String::as_str)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    assert_eq!(output.suggestions[0].status, "noMatch");
    assert!(output.edits.is_empty());
    assert_eq!(update_arguments, [vec!["update", "14.0.0-beta.1"]]);
}

#[test]
fn show_prereleases_applies_to_composer_update_choices() {
    let session = VersionLensSession::new(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: Vec::new(),
        providers: ProviderSettings::default(),
        suggestion_indicators: SuggestionIndicators::standard(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: true,
        http: HttpConfig::standard(),
    });

    let input = DocumentInput {
        uri: "file:///repo/composer.json".to_owned(),
        language_id: "json".to_owned(),
        text: r#"{"require":{"php-parallel-lint/php-parallel-lint":"3.1.3"}}"#.to_owned(),
        workspace_root: None,
    };

    session.resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput {
            package: "php-parallel-lint/php-parallel-lint".to_owned(),
            ecosystem: Ecosystem::Composer,
            body: r#"{
              "packages": {
                "php-parallel-lint/php-parallel-lint": [
                  { "version": "v3.1.3" },
                  { "version": "v3.2.0-beta.1" }
                ]
              }
            }"#
            .to_owned(),
        }],
    );

    let analysis = session.analyze_document(input);

    assert!(
        analysis
            .code_lenses
            .iter()
            .any(|lens| lens.title == "↑  beta 3.2.0-beta.1")
    );
}

#[test]
fn show_prereleases_applies_to_npm_versions() {
    let session = VersionLensSession::new(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: Vec::new(),
        providers: ProviderSettings::default(),
        suggestion_indicators: SuggestionIndicators::standard(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: true,
        http: HttpConfig::standard(),
    });

    let input = DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: r#"{"dependencies":{"typescript":"6.0.0"}}"#.to_owned(),
        workspace_root: None,
    };
    let responses = [RegistryResponseInput {
        package: "typescript".to_owned(),
        ecosystem: Ecosystem::Npm,
        body: r#"{"dist-tags":{"latest":"6.0.3"},"versions":{"6.0.3":{},"7.0.0-beta.1":{}}}"#
            .to_owned(),
    }];

    session.resolve_document_with_responses(input.clone(), &responses);
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

    assert_eq!(
        arguments,
        [vec!["update", "6.0.3"], vec!["update", "7.0.0-beta.1"]]
    );
}

#[test]
fn show_prereleases_keeps_npm_prerelease_choice_when_fixed_version_is_latest() {
    let session = VersionLensSession::new(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: Vec::new(),
        providers: ProviderSettings::default(),
        suggestion_indicators: SuggestionIndicators::standard(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: true,
        http: HttpConfig::standard(),
    });

    let input = DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: r#"{"dependencies":{"left-pad":"3.0.0"}}"#.to_owned(),
        workspace_root: None,
    };
    let responses = [RegistryResponseInput {
        package: "left-pad".to_owned(),
        ecosystem: Ecosystem::Npm,
        body: r#"{
          "dist-tags": { "latest": "3.0.0" },
          "versions": {
            "1.0.0": {},
            "1.1.0-alpha.1": {},
            "2.0.0": {},
            "2.1.0": {},
            "3.0.0": {},
            "4.0.0-next": {}
          }
        }"#
        .to_owned(),
    }];

    let resolved = session.resolve_document_with_responses(input.clone(), &responses);
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
                .map(String::as_str)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    assert_eq!(resolved.suggestions[0].status, "current");
    assert_eq!(resolved.suggestions[0].latest.as_deref(), Some("3.0.0"));
    assert!(resolved.edits.is_empty());
    assert_eq!(titles, ["🟢 latest 3.0.0", "↑  next 4.0.0-next"]);
    assert_eq!(arguments, [vec!["update", "4.0.0-next"]]);
}

#[test]
fn prerelease_tag_filters_apply_to_responses() {
    let session = VersionLensSession::new(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: Vec::new(),
        providers: ProviderSettings {
            prerelease_tags: vec![PrereleaseTagConfig {
                ecosystem: Ecosystem::Npm,
                tags: vec!["beta".to_owned()],
            }],
            ..ProviderSettings::default()
        },
        suggestion_indicators: SuggestionIndicators::standard(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: true,
        http: HttpConfig::standard(),
    });

    let input = DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: r#"{"dependencies":{"typescript":"6.0.0"}}"#.to_owned(),
        workspace_root: None,
    };
    let responses = [RegistryResponseInput {
        package: "typescript".to_owned(),
        ecosystem: Ecosystem::Npm,
        body: r#"{"dist-tags":{"latest":"6.0.3"},"versions":{"6.0.3":{},"7.0.0-beta.1":{},"8.0.0-rc.1":{}}}"#
            .to_owned(),
    }];

    session.resolve_document_with_responses(input.clone(), &responses);
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

    assert_eq!(
        arguments,
        [vec!["update", "6.0.3"], vec!["update", "7.0.0-beta.1"]]
    );
}

#[test]
fn prerelease_ranges_can_resolve_prerelease_versions_when_hidden() {
    let session = VersionLensSession::new(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: Vec::new(),
        providers: ProviderSettings::default(),
        suggestion_indicators: SuggestionIndicators::standard(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: HttpConfig::standard(),
    });

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: r#"{"dependencies":{"typescript":"^1.0.0-beta.1"}}"#.to_owned(),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "typescript".to_owned(),
            ecosystem: Ecosystem::Npm,
            body: r#"{"versions":{"2.0.0-beta.1":{}}}"#.to_owned(),
        }],
    );

    assert_eq!(output.edits[0].new_text, "^2.0.0-beta.1");
}

#[test]
fn show_prereleases_applies_to_python_releases() {
    let session = VersionLensSession::new(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: Vec::new(),
        providers: ProviderSettings::default(),
        suggestion_indicators: SuggestionIndicators::standard(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: true,
        http: HttpConfig::standard(),
    });

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///requirements.txt".to_owned(),
            language_id: "pip-requirements".to_owned(),
            text: "flask==3.0.0".to_owned(),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "flask".to_owned(),
            ecosystem: Ecosystem::Python,
            body: r#"{"info":{"version":"3.0.0"},"releases":{"3.0.0":[],"4.0.0rc1":[]}}"#
                .to_owned(),
        }],
    );

    assert_eq!(output.edits[0].new_text, "==4.0.0rc1");
}
