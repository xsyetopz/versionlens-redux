use super::{
    SessionConfig, SuggestionIndicators, dependency_property_config_from_name,
    dependency_property_manifest_kind_from_name, enabled_provider_config_from_name,
    file_pattern_config_from_name, prerelease_tag_config_from_name,
    provider_cache_config_from_name, provider_http_config_from_name,
    provider_settings_manifest_kind_from_name, registry_url_config_from_name,
};
use versionlens_model::Ecosystem::*;
use versionlens_model::ManifestKind::{
    Cabal, DenoImportMapJson, DenoJson, GitHubActions, JsrJson, MixExs, NpmPackageJson,
    NpmPackageJson5, NpmPackageYaml, Opam, PnpmYaml,
};

#[test]
fn enabled_provider_config_names_are_manifest_scoped() {
    let npm = enabled_provider_config_from_name("npm").expect("npm provider");
    assert_eq!(npm.ecosystem, Npm);
    assert_eq!(npm.manifest_kind, Some(NpmPackageJson));

    let bun = enabled_provider_config_from_name("bun").expect("bun provider");
    assert_eq!(bun.ecosystem, Npm);
    assert_eq!(bun.manifest_kind, Some(NpmPackageJson));

    let pnpm = enabled_provider_config_from_name("pnpm").expect("pnpm provider");
    assert_eq!(pnpm.ecosystem, Npm);
    assert_eq!(pnpm.manifest_kind, Some(PnpmYaml));

    let cargo = enabled_provider_config_from_name("cargo").expect("cargo provider");
    assert_eq!(cargo.ecosystem, Cargo);
    assert_eq!(cargo.manifest_kind, None);

    let github = enabled_provider_config_from_name("github").expect("github provider");
    assert_eq!(github.ecosystem, GitHub);
    assert_eq!(github.manifest_kind, Some(GitHubActions));

    assert_eq!(enabled_provider_config_from_name("unknown"), None);
}

#[test]
fn npm_enabled_provider_config_applies_to_package_json5_and_package_yaml() {
    let config = enabled_provider_config_from_name("npm").expect("npm provider");

    assert!(config.applies_to_manifest(NpmPackageJson, Npm));
    assert!(config.applies_to_manifest(NpmPackageJson5, Npm));
    assert!(config.applies_to_manifest(NpmPackageYaml, Npm));
    assert!(!config.applies_to_manifest(PnpmYaml, Npm));
}

#[test]
fn deno_enabled_provider_config_applies_to_import_maps_and_jsr_config() {
    let config = enabled_provider_config_from_name("deno").expect("deno provider");

    assert!(config.applies_to_manifest(DenoJson, Deno));
    assert!(config.applies_to_manifest(DenoImportMapJson, Deno));
    assert!(config.applies_to_manifest(JsrJson, Deno));
    assert!(!config.applies_to_manifest(NpmPackageJson, Npm));
}

#[test]
fn dependency_property_names_are_manifest_scoped() {
    assert_eq!(
        dependency_property_manifest_kind_from_name("npm"),
        Some(NpmPackageJson)
    );
    assert_eq!(
        dependency_property_manifest_kind_from_name("bun"),
        Some(NpmPackageJson)
    );
    assert_eq!(
        dependency_property_manifest_kind_from_name("pnpm"),
        Some(PnpmYaml)
    );
    assert_eq!(dependency_property_manifest_kind_from_name("cargo"), None);
    assert_eq!(
        dependency_property_manifest_kind_from_name("github"),
        Some(GitHubActions)
    );
}

#[test]
fn provider_setting_names_are_manifest_scoped() {
    assert_eq!(
        provider_settings_manifest_kind_from_name("pnpm"),
        Some(PnpmYaml)
    );
    assert_eq!(provider_settings_manifest_kind_from_name("npm"), None);
    assert_eq!(provider_settings_manifest_kind_from_name("cargo"), None);
    assert_eq!(
        provider_settings_manifest_kind_from_name("github"),
        Some(GitHubActions)
    );
}

#[test]
fn github_file_pattern_config_routes_to_github_actions() {
    let config = file_pattern_config_from_name(
        "github",
        " **/.github/{workflows,actions}/**/*.{yml,yaml} ".to_owned(),
    )
    .expect("GitHub Actions file pattern config");

    assert_eq!(config.manifest_kind, GitHubActions);
    assert_eq!(
        config.pattern,
        "**/.github/{workflows,actions}/**/*.{yml,yaml}"
    );
}

#[test]
fn dependency_property_config_uses_provider_scope_override() {
    let config =
        dependency_property_config_from_name("npm", Some("pnpm"), vec!["catalog".to_owned()])
            .expect("dependency property config");

    assert_eq!(config.ecosystem, Npm);
    assert_eq!(config.manifest_kind, Some(PnpmYaml));
    assert_eq!(config.properties, ["catalog"]);
}

#[test]
fn hex_file_pattern_config_routes_to_mix_exs() {
    let config = file_pattern_config_from_name("hex", " **/mix.exs ".to_owned())
        .expect("hex file pattern config");

    assert_eq!(config.manifest_kind, MixExs);
    assert_eq!(config.pattern, "**/mix.exs");
}

#[test]
fn opam_file_pattern_config_routes_to_opam() {
    let config = file_pattern_config_from_name("opam", " **/{opam,*.opam} ".to_owned())
        .expect("opam file pattern config");

    assert_eq!(config.manifest_kind, Opam);
    assert_eq!(config.pattern, "**/{opam,*.opam}");
}

#[test]
fn hackage_file_pattern_config_routes_to_cabal() {
    let config = file_pattern_config_from_name(
        "hackage",
        " **/{*.cabal,cabal.project,stack.yaml} ".to_owned(),
    )
    .expect("hackage file pattern config");

    assert_eq!(config.manifest_kind, Cabal);
    assert_eq!(config.pattern, "**/{*.cabal,cabal.project,stack.yaml}");
}

#[test]
fn registry_url_config_trims_urls_and_rejects_blank_values() {
    let config = registry_url_config_from_name("cargo", " https://mirror.test/crates ".to_owned())
        .expect("registry url config");

    assert_eq!(config.ecosystem, Cargo);
    assert_eq!(config.url, "https://mirror.test/crates");
    assert_eq!(
        registry_url_config_from_name("cargo", "   ".to_owned()),
        None
    );
}

#[test]
fn prerelease_tag_config_trims_tags_and_rejects_empty_results() {
    let config = prerelease_tag_config_from_name(
        "npm",
        vec![" beta ".to_owned(), "".to_owned(), "  ".to_owned()],
    )
    .expect("prerelease tag config");

    assert_eq!(config.ecosystem, Npm);
    assert_eq!(config.tags, ["beta"]);
    assert_eq!(
        prerelease_tag_config_from_name("npm", vec!["".to_owned()]),
        None
    );
}

#[test]
fn provider_cache_config_maps_duration_and_manifest_scope() {
    let config =
        provider_cache_config_from_name("pnpm", Some(0.25)).expect("provider cache config");

    assert_eq!(config.ecosystem, Npm);
    assert_eq!(config.manifest_kind, Some(PnpmYaml));
    assert_eq!(config.cache_ttl_ms, 15_000);
    assert_eq!(provider_cache_config_from_name("pnpm", None), None);
}

#[test]
fn provider_http_config_maps_manifest_scope() {
    let config = provider_http_config_from_name("pnpm", Some(false)).expect("provider http config");

    assert_eq!(config.ecosystem, Npm);
    assert_eq!(config.manifest_kind, Some(PnpmYaml));
    assert_eq!(config.strict_ssl, Some(false));
}

#[test]
fn session_config_default_applies_core_defaults() {
    let config = SessionConfig::default();

    assert_eq!(config.cache_ttl_ms, 180_000);
    assert!(config.enabled_providers.is_empty());
    assert_eq!(config.providers.registry_urls, []);
    assert_eq!(config.suggestion_indicators.updateable, "\u{2191} ");
    assert!(config.show_vulnerabilities);
    assert!(!config.show_suggestion_stats);
    assert!(!config.show_prereleases);
    assert_eq!(config.http.timeout_ms, 10_000);
    assert!(config.http.strict_ssl);
}

#[test]
fn suggestion_indicators_replace_blank_values_with_standard_indicators() {
    let defaults = crate::standard_suggestion_indicators();

    for blank in ["", " \t\n\u{2003} "] {
        let indicators = SuggestionIndicators {
            latest: blank.to_owned(),
            satisfies_latest: blank.to_owned(),
            directory: blank.to_owned(),
            error: blank.to_owned(),
            no_match: blank.to_owned(),
            matched: blank.to_owned(),
            updateable: blank.to_owned(),
            updateable_vulnerable: blank.to_owned(),
            build: blank.to_owned(),
        }
        .with_standard_indicators_for_blanks();

        assert_eq!(indicators, defaults);
    }
}

#[test]
fn suggestion_indicators_preserve_nonblank_custom_values_verbatim() {
    let indicators = SuggestionIndicators {
        latest: " latest ".to_owned(),
        satisfies_latest: " satisfies ".to_owned(),
        directory: " directory ".to_owned(),
        error: " error ".to_owned(),
        no_match: " no-match ".to_owned(),
        matched: " matched ".to_owned(),
        updateable: " update ".to_owned(),
        updateable_vulnerable: " vulnerable ".to_owned(),
        build: " build ".to_owned(),
    }
    .with_standard_indicators_for_blanks();

    assert_eq!(indicators.latest, " latest ");
    assert_eq!(indicators.satisfies_latest, " satisfies ");
    assert_eq!(indicators.directory, " directory ");
    assert_eq!(indicators.error, " error ");
    assert_eq!(indicators.no_match, " no-match ");
    assert_eq!(indicators.matched, " matched ");
    assert_eq!(indicators.updateable, " update ");
    assert_eq!(indicators.updateable_vulnerable, " vulnerable ");
    assert_eq!(indicators.build, " build ");
}

#[test]
fn direct_session_config_preserves_nonblank_custom_indicators() {
    let indicators = crate::SuggestionIndicators {
        latest: " latest ".to_owned(),
        satisfies_latest: " satisfies ".to_owned(),
        directory: " directory ".to_owned(),
        error: " error ".to_owned(),
        no_match: " no-match ".to_owned(),
        matched: " matched ".to_owned(),
        updateable: " update ".to_owned(),
        updateable_vulnerable: " vulnerable ".to_owned(),
        build: " build ".to_owned(),
    };
    let session = crate::version_lens_session(SessionConfig {
        update_severity: Some(crate::Severity::Information),
        cache_ttl_ms: 1,
        enabled_providers: vec![],
        providers: crate::default(),
        suggestion_indicators: indicators.clone(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_fetch::standard_http_config(),
    });

    assert_eq!(session.config.suggestion_indicators, indicators);
}
