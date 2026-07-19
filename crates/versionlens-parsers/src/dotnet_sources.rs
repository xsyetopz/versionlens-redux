mod nuget_config;
mod parse;
mod protocol;
mod remote;
mod schema;

pub use nuget_config::{
    parse_nuget_config, parse_nuget_config_auth_entries, parse_nuget_config_named_sources,
    parse_nuget_config_source_mappings, parse_nuget_config_source_urls,
};
pub use parse::{parse_dotnet_enabled_sources, parse_dotnet_sources};
pub use remote::filter_dotnet_remote_sources;
pub use schema::{
    DotnetAuthEntry, DotnetNamedSource, DotnetNugetConfig, DotnetSource, DotnetSourceMapping,
};

#[cfg(test)]
mod tests;
