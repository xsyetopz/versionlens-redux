use crate::dependency::properties::is_enabled;
use versionlens_model::{
    Dependency, DocumentInput, Ecosystem, ManifestKind, ecosystem_for_manifest,
};
use versionlens_parsers::parse_document_as_manifest_with_dependency_paths;

use crate::DependencyPropertyConfig;
use crate::VersionLensSession;
use versionlens_cache::CacheKey;

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
            .unwrap_or_else(crate::recover_poison);
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
mod tests;
