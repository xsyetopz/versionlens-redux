use versionlens_parsers::Dependency;

use crate::VersionLensSession;
use crate::dotnet_sources::runtime_dotnet_registry_source_urls;

impl VersionLensSession {
    pub(in crate::registry::urls) fn dotnet_registry_urls(
        &self,
        _dependency: &Dependency,
    ) -> Vec<String> {
        {
            let mut source_cache = self
                .dotnet_registry_sources
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            source_cache
                .get_or_insert_with(|| {
                    runtime_dotnet_registry_source_urls(&self.config.providers.registry_urls)
                })
                .iter()
                .map(String::as_str)
                .map(str::to_owned)
                .collect::<Vec<_>>()
        }
    }
}
