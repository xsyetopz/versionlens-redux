use versionlens_parsers::{
    Dependency, DocumentInput, Ecosystem, ManifestKind, ecosystem_for_manifest,
    parse_document_as_manifest_with_dependency_paths,
};

use crate::DependencyPropertyConfig;
use crate::VersionLensSession;
use crate::dependency::properties;

impl VersionLensSession {
    pub(crate) fn dependencies(&self, input: &DocumentInput) -> Vec<Dependency> {
        let kind = self.classify_document(input);
        if let Some(ecosystem) = ecosystem_for_manifest(kind)
            && !self.provider_enabled_for_manifest(kind, ecosystem)
        {
            return Vec::new();
        }

        let dependency_paths = self.dependency_paths_for_manifest(kind);

        parse_document_as_manifest_with_dependency_paths(input, kind, &dependency_paths)
            .into_iter()
            .filter(|dependency| self.dependency_property_enabled(dependency, kind))
            .collect()
    }

    fn dependency_paths_for_manifest(&self, kind: ManifestKind) -> Vec<&str> {
        let Some(ecosystem) = ecosystem_for_manifest(kind) else {
            return Vec::new();
        };

        self.dependency_property_configs(kind, ecosystem)
            .flat_map(|config| config.properties.iter().map(String::as_str))
            .collect()
    }

    fn dependency_property_enabled(&self, dependency: &Dependency, kind: ManifestKind) -> bool {
        let Some(ecosystem) = ecosystem_for_manifest(kind) else {
            return true;
        };

        properties::is_enabled(
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
