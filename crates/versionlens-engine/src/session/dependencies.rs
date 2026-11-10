use crate::dependency::properties::is_enabled;
use std::sync::PoisonError;
use versionlens_model::{Dependency, DocumentInput, Ecosystem, ManifestKind};
use versionlens_parsers::ecosystem_for_manifest;
use versionlens_parsers::parse_document_as_manifest_with_dependency_paths;

use crate::DependencyPropertyConfig;
use crate::VersionLensSession;
use versionlens_fetch::CacheKey;

impl VersionLensSession {
    pub(crate) fn dependencies(&self, input: &DocumentInput) -> Vec<Dependency> {
        let kind = self.classify_document(input);
        if let Some(ecosystem) = ecosystem_for_manifest(kind)
            && !self.provider_enabled_for_manifest(kind, ecosystem)
        {
            return vec![];
        }

        let dependency_paths = self.dependency_paths_for_manifest(kind);
        let key = serde_json::to_vec(&(
            &input.uri,
            &input.language_id,
            &input.text,
            &input.workspace_root,
            kind,
            &self.config.providers.dependency_properties,
        ))
        .ok()
        .map(|identity| CacheKey::content(&identity));
        let mut cache = self
            .result_state
            .parsed_dependencies
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        if let Some(key) = &key
            && let Some(dependencies) = cache.get(key)
        {
            return dependencies.clone();
        }
        let dependencies: Vec<_> =
            parse_document_as_manifest_with_dependency_paths(input, kind, &dependency_paths)
                .into_iter()
                .filter(|dependency| self.dependency_property_enabled(dependency, kind))
                .collect();
        if let Some(key) = key {
            cache.insert(key, dependencies.clone());
        }
        dependencies
    }

    /// Dependencies plus the project version fields that only the
    /// project-version commands edit, such as the npm and Cargo `version`.
    pub(crate) fn command_dependencies(&self, input: &DocumentInput) -> Vec<Dependency> {
        let mut dependencies = self.dependencies(input);
        let kind = self.classify_document(input);
        if ecosystem_for_manifest(kind)
            .is_none_or(|ecosystem| self.provider_enabled_for_manifest(kind, ecosystem))
        {
            dependencies.extend(versionlens_parsers::parse_project_versions_as_manifest(
                input, kind,
            ));
        }
        dependencies
    }

    fn dependency_paths_for_manifest(&self, kind: ManifestKind) -> Vec<&str> {
        let Some(ecosystem) = ecosystem_for_manifest(kind) else {
            return vec![];
        };

        self.dependency_property_configs(kind, ecosystem)
            .flat_map(|config| config.properties.iter().map(|value| value.as_str()))
            .collect()
    }

    fn dependency_property_enabled(&self, dependency: &Dependency, kind: ManifestKind) -> bool {
        let Some(ecosystem) = ecosystem_for_manifest(kind) else {
            return true;
        };

        is_enabled(
            dependency,
            ecosystem,
            self.dependency_property_configs(kind, ecosystem),
        )
    }

    fn dependency_property_configs(
        &self,
        kind: ManifestKind,
        ecosystem: Ecosystem,
    ) -> impl Iterator<Item = &DependencyPropertyConfig> {
        self.config
            .providers
            .dependency_properties
            .iter()
            .filter(move |config| config.ecosystem == ecosystem && config.applies_to_manifest(kind))
    }
}

#[cfg(test)]
mod tests {
use versionlens_model::DocumentInput;

use crate::{
    EnabledProviderConfig, FilePatternConfig, ProviderSettings, SessionConfig, VersionLensSession,
};
use versionlens_model::Ecosystem::*;
use versionlens_model::ManifestKind::{
    Cabal, ComposerJson, DenoJson, DockerComposeYaml, DubJson, Gemfile, GleamToml,
    JuliaProjectToml, MixExs, NpmPackageJson, Opam, PnpmYaml, PythonRequirementsTxt, RDescription,
    RebarConfig,
};

include!("tests/providers.rs");
include!("tests/patterns.rs");

#[test]
fn changed_snapshot_reconstructs_requirements_and_unicode_ranges() {
    let session = crate::test_support::test_session(false);
    let initial = DocumentInput::new(
        "file:///package.json",
        "json",
        r#"{"dependencies":{"example":"1.0.0"}}"#,
        None,
    );
    let first = session.dependencies(&initial);
    let mut changed = initial.clone();
    changed.text =
        "{\n  \"description\": \"🦀\", \"dependencies\": {\"example\": \"2.0.0\"}\n}".to_owned();
    let second = session.dependencies(&changed);
    assert_eq!(first[0].requirement, "1.0.0");
    assert_eq!(second[0].requirement, "2.0.0");
    assert_eq!(second[0].requirement_range.start.line, 1);
    assert_ne!(first[0].requirement_range, second[0].requirement_range);
    assert_eq!(session.dependencies(&initial), first);
    assert_eq!(session.dependencies(&changed), second);
}

#[test]
fn configured_patterns_match_decoded_workspace_paths() {
    let session = session_with_file_pattern(FilePatternConfig {
        manifest_kind: NpmPackageJson,
        pattern: "ci files/dépendances.custom".to_owned(),
    });
    for root in [
        "/workspace/project files",
        "file:///workspace/project%20files",
    ] {
        let input = DocumentInput {
            uri: "file:///workspace/project%20files/ci%20files/d%C3%A9pendances.custom".to_owned(),
            language_id: "plaintext".to_owned(),
            text: r#"{"dependencies":{"example":"1.0.0"}}"#.to_owned(),
            workspace_root: Some(root.to_owned()),
            version: None,
        };
        let analysis = session.analyze_document(input);
        assert!(analysis.is_supported_manifest);
        assert_eq!(analysis.dependencies.len(), 1);
        assert_eq!(analysis.dependencies[0].name, "example");
    }
}
fn session_with_enabled_provider(provider: EnabledProviderConfig) -> VersionLensSession {
    crate::version_lens_session(SessionConfig {
        update_severity: Some(crate::Severity::Information),
        cache_ttl_ms: 300_000,
        enabled_providers: vec![provider],
        providers: ProviderSettings::default(),
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_fetch::standard_http_config(),
    })
}

fn session_with_file_pattern(file_pattern: FilePatternConfig) -> VersionLensSession {
    crate::version_lens_session(crate::test_support::session_config(
        ProviderSettings {
            file_patterns: vec![file_pattern],
            ..Default::default()
        },
        true,
    ))
}

fn package_file_fixture(name: &str) -> String {
    crate::test_support::fixture("tests/fixtures/session/dependencies", name)
}
}
