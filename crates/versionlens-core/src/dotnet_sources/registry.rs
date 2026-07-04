use versionlens_parsers::{Ecosystem, parse_dotnet_enabled_sources};

use crate::config::RegistryUrlConfig;

pub fn dotnet_registry_source_urls(
    configured: &[RegistryUrlConfig],
    dotnet_sources: Option<&str>,
) -> Vec<String> {
    let feed_urls = configured
        .iter()
        .filter(|url| url.ecosystem == Ecosystem::Dotnet)
        .map(|url| url.url.as_str().to_owned())
        .collect::<Vec<_>>();

    parse_dotnet_enabled_sources(dotnet_sources.unwrap_or_default(), &feed_urls)
        .into_iter()
        .map(|source| source.url)
        .collect()
}

#[cfg(test)]
mod tests;
