use std::fs::read_to_string;
use std::path::PathBuf;

use versionlens_model::{DocumentInput, Ecosystem, ManifestKind};

use crate::{DependencyPropertyConfig, ProviderSettings, SessionConfig, VersionLensSession};

fn session_with_properties(ecosystem: Ecosystem, properties: &[&str]) -> VersionLensSession {
    session_with_property_configs(&[(ecosystem, properties)])
}

fn session_with_property_configs(configs: &[(Ecosystem, &[&str])]) -> VersionLensSession {
    session_with_scoped_property_configs(
        &configs
            .iter()
            .map(|(ecosystem, properties)| (*ecosystem, None, *properties))
            .collect::<Vec<_>>(),
    )
}

fn session_with_scoped_property_configs(
    configs: &[(Ecosystem, Option<ManifestKind>, &[&str])],
) -> VersionLensSession {
    crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: ProviderSettings {
            dependency_properties: configs
                .iter()
                .map(
                    |(ecosystem, manifest_kind, properties)| DependencyPropertyConfig {
                        ecosystem: *ecosystem,
                        manifest_kind: *manifest_kind,
                        properties: properties
                            .iter()
                            .map(|property| (*property).to_owned())
                            .collect(),
                    },
                )
                .collect(),
            ..crate::default()
        },
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    })
}

fn package_file_fixture(name: &str) -> String {
    let path = repo_root()
        .join("tests/fixtures/dependency-properties")
        .join(name);
    read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "failed to read dependency-property fixture {}: {error}",
            path.display()
        )
    })
}

fn repo_root() -> PathBuf {
    let manifest_dir: PathBuf = env!("CARGO_MANIFEST_DIR").into();
    manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .expect("core crate should be under crates/")
        .to_path_buf()
}

mod cargo;
mod npm;
mod pub_manifest;
mod python;
mod xml;
