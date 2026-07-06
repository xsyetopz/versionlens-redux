use crate::VersionLensSession;
use crate::dotnet_sources::runtime_dotnet_registry_source_urls;

impl VersionLensSession {
    pub(in crate::registry::urls) fn dotnet_registry_urls(&self) -> Vec<String> {
        {
            let mut source_cache = self
                .dotnet_registry_sources
                .lock()
                .unwrap_or_else(|poisoned| crate::recover_poison(poisoned));
            source_cache
                .get_or_insert_with(|| {
                    runtime_dotnet_registry_source_urls(&self.config.providers.registry_urls)
                })
                .iter()
                .map(|value| value.as_str())
                .map(|value| value.to_owned())
                .collect::<Vec<_>>()
        }
    }
}
