mod listing;
mod registry;

use crate::config::RegistryUrlConfig;

use listing::dotnet_source_listing;
pub use registry::dotnet_registry_source_urls;

pub(crate) fn runtime_dotnet_registry_source_urls(configured: &[RegistryUrlConfig]) -> Vec<String> {
    let source_listing = dotnet_source_listing();
    dotnet_registry_source_urls(configured, source_listing.as_deref())
}
