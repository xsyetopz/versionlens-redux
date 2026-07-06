use versionlens_http::{HttpConfigInput, HttpHeaderInput};

use super::{
    DependencyPropertyConfigInput, FilePatternConfigInput, PrereleaseTagConfigInput,
    ProviderCacheConfigInput, ProviderHttpConfigInput, ProviderSettingsInput,
    RegistryUrlConfigInput, SessionConfig, SessionConfigInput, SuggestionIndicatorsInput,
    dependency_property_config_from_name, dependency_property_manifest_kind_from_name,
    enabled_provider_config_from_name, file_pattern_config_from_name,
    prerelease_tag_config_from_name, provider_cache_config_from_name,
    provider_http_config_from_name, provider_settings_manifest_kind_from_name,
    registry_url_config_from_name,
};
use versionlens_parsers::Ecosystem::{Cargo, Deno, Npm};
use versionlens_parsers::ManifestKind::{
    Cabal, ComposerJson, DenoImportMapJson, DenoJson, JsrJson, MixExs, NpmPackageJson,
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
}

#[test]
fn provider_setting_names_are_manifest_scoped() {
    assert_eq!(
        provider_settings_manifest_kind_from_name("pnpm"),
        Some(PnpmYaml)
    );
    assert_eq!(provider_settings_manifest_kind_from_name("npm"), None);
    assert_eq!(provider_settings_manifest_kind_from_name("cargo"), None);
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
fn session_config_input_applies_core_defaults() {
    let config = crate::session_config_from_input(SessionConfigInput {
        cache_duration_minutes: None,
        cache_ttl_seconds: None,
        enabled_providers: None,
        providers: None,
        suggestion_indicators: None,
        show_vulnerabilities: None,
        show_suggestion_stats: None,
        show_prereleases: false,
        http: None,
    });

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
fn session_config_input_normalizes_session_indicator_and_http_values() {
    let config = normalized_session_config();

    assert_eq!(config.cache_ttl_ms, 30_000);
    assert_eq!(config.enabled_providers.len(), 2);
    assert_eq!(config.enabled_providers[0].ecosystem, Cargo);
    assert_eq!(config.enabled_providers[1].ecosystem, Npm);
    assert_eq!(config.enabled_providers[1].manifest_kind, Some(PnpmYaml));
    assert_eq!(config.suggestion_indicators.updateable, "U");
    assert_eq!(config.suggestion_indicators.latest, "\u{1F7E2}");
    assert!(!config.show_vulnerabilities);
    assert!(config.show_suggestion_stats);
    assert!(config.show_prereleases);
    assert_eq!(config.http.timeout_ms, 250);
    assert!(!config.http.strict_ssl);
    assert_eq!(config.http.proxy.as_deref(), Some("http://localhost:8080"));
    assert_eq!(config.http.auth_headers.len(), 1);
    assert_eq!(config.http.auth_headers[0].name, "authorization");
}

#[test]
fn session_config_input_normalizes_provider_values() {
    let config = normalized_session_config();

    assert_eq!(config.providers.registry_urls.len(), 1);
    assert_eq!(
        config.providers.registry_urls[0].url,
        "https://mirror.test/crates"
    );
    assert_eq!(config.providers.prerelease_tags[0].tags, ["beta"]);
    assert_eq!(config.providers.provider_cache[0].cache_ttl_ms, 15_000);
    assert_eq!(
        config.providers.provider_http[0].manifest_kind,
        Some(PnpmYaml)
    );
    assert_eq!(
        config.providers.dependency_properties[0].manifest_kind,
        Some(PnpmYaml)
    );
    assert_eq!(
        config.providers.file_patterns[0].manifest_kind,
        ComposerJson
    );
    assert_eq!(
        config.providers.file_patterns[0].pattern,
        "**/acme.composer.json"
    );
}

fn normalized_session_config() -> SessionConfig {
    crate::session_config_from_input(SessionConfigInput {
        cache_duration_minutes: Some(0.5),
        cache_ttl_seconds: Some(90),
        enabled_providers: Some(vec![
            "cargo".to_owned(),
            "unknown".to_owned(),
            "pnpm".to_owned(),
        ]),
        providers: Some(ProviderSettingsInput {
            registry_urls: Some(vec![
                RegistryUrlConfigInput {
                    ecosystem: "cargo".to_owned(),
                    url: "   ".to_owned(),
                },
                RegistryUrlConfigInput {
                    ecosystem: "cargo".to_owned(),
                    url: " https://mirror.test/crates ".to_owned(),
                },
            ]),
            prerelease_tag_filters: Some(vec![PrereleaseTagConfigInput {
                ecosystem: "npm".to_owned(),
                tags: vec![" beta ".to_owned(), " ".to_owned()],
            }]),
            provider_cache: Some(vec![ProviderCacheConfigInput {
                ecosystem: "pnpm".to_owned(),
                cache_duration_minutes: Some(0.25),
            }]),
            provider_http: Some(vec![ProviderHttpConfigInput {
                ecosystem: "pnpm".to_owned(),
                strict_ssl: Some(false),
            }]),
            dependency_properties: Some(vec![DependencyPropertyConfigInput {
                ecosystem: "npm".to_owned(),
                provider: Some("pnpm".to_owned()),
                properties: vec!["catalog".to_owned()],
            }]),
            file_patterns: Some(vec![FilePatternConfigInput {
                ecosystem: "composer".to_owned(),
                pattern: " **/acme.composer.json ".to_owned(),
            }]),
        }),
        suggestion_indicators: Some(SuggestionIndicatorsInput {
            updateable: Some("U".to_owned()),
            ..crate::default()
        }),
        show_vulnerabilities: Some(false),
        show_suggestion_stats: Some(true),
        show_prereleases: true,
        http: Some(HttpConfigInput {
            timeout_ms: Some(250),
            strict_ssl: Some(false),
            proxy: Some(" http://localhost:8080 ".to_owned()),
            ca_file: None,
            ca: None,
            cert_file: None,
            key_file: None,
            cert: None,
            key: None,
            auth_headers: Some(vec![
                HttpHeaderInput {
                    name: " ".to_owned(),
                    value: "ignored".to_owned(),
                    url: None,
                },
                HttpHeaderInput {
                    name: " authorization ".to_owned(),
                    value: " Bearer token ".to_owned(),
                    url: Some(" https://registry.example.test ".to_owned()),
                },
            ]),
        }),
    })
}
