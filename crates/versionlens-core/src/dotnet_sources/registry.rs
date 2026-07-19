use versionlens_parsers::parse_dotnet_enabled_sources;

use crate::config::RegistryUrlConfig;
use versionlens_model::Ecosystem::Dotnet;

pub fn dotnet_registry_source_urls(
    configured: &[RegistryUrlConfig],
    dotnet_sources: Option<&str>,
) -> Vec<String> {
    let feed_urls = configured
        .iter()
        .filter(|url| url.ecosystem == Dotnet)
        .map(|url| url.url.as_str().to_owned())
        .collect::<Vec<_>>();

    parse_dotnet_enabled_sources(dotnet_sources.unwrap_or_default(), &feed_urls)
        .into_iter()
        .map(|source| source.url)
        .collect()
}

#[cfg(test)]
mod tests;
