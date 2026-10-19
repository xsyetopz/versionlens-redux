use versionlens_model::{DocumentInput, Ecosystem, ManifestKind};

use crate::{DependencyPropertyConfig, ProviderSettings, VersionLensSession};

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
    crate::version_lens_session(crate::support::tests::session_config(
        ProviderSettings {
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
        true,
    ))
}

fn package_file_fixture(name: &str) -> String {
    crate::support::tests::fixture("tests/fixtures/dependency-properties", name)
}

mod cargo;
mod npm;
mod pub_manifest;
mod python;
mod xml;
