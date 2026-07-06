use anyhow::Error as AnyhowError;
use semver::{Error as SemverError, Version as SemverVersion, VersionReq as SemverVersionReq};
use std::error::Error as StdError;
#[cfg(test)]
use std::io::Result as IoResult;
#[cfg(test)]
use std::net::TcpListener as StdTcpListener;
use std::path::Path as StdPath;
use std::string::FromUtf8Error as StringFromUtf8Error;
use std::sync::Arc as StdArc;
use std::sync::{Mutex as StdMutex, PoisonError as SyncPoisonError};
use std::time::Duration as StdDuration;
#[cfg(test)]
use std::time::SystemTime as StdSystemTime;
use versionlens_cache::MemoryCache as VersionLensMemoryCache;
mod cache;
mod command;
mod concurrency;
mod config;
mod dependency;
mod docker;
mod dotnet_sources;
mod error;
mod fetch;
mod model;
mod non_registry;
mod prerelease;
mod presentation;
mod project;
mod registry;
mod runtime_config;
mod schema;
mod selection;
mod session;
mod snapshot;
mod status;
mod suggestion;
mod vulnerability;

pub use config::{
    DependencyPropertyConfig, DependencyPropertyConfigInput, EnabledProviderConfig,
    FilePatternConfig, FilePatternConfigInput, PrereleaseTagConfig, PrereleaseTagConfigInput,
    ProviderCacheConfig, ProviderCacheConfigInput, ProviderHttpConfig, ProviderHttpConfigInput,
    ProviderSettings, ProviderSettingsInput, RegistryUrlConfig, RegistryUrlConfigInput,
    SessionConfig, SessionConfigInput, SuggestionIndicators, SuggestionIndicatorsInput,
    dependency_property_config_from_name, dependency_property_manifest_kind_from_name,
    enabled_provider_config_from_name, file_pattern_config_from_name,
    file_pattern_manifest_kind_from_name, prerelease_tag_config_from_name,
    provider_cache_config_from_input, provider_cache_config_from_name,
    provider_http_config_from_name, provider_settings_manifest_kind_from_name,
    registry_url_config_from_name, standard_suggestion_indicators,
};
pub use dependency::dependency_payload;
pub use dotnet_sources::dotnet_registry_source_urls;
pub use model::{
    AnalyzeDocumentOutput, AuthorizationRequestPayload, RegistryResponseInput,
    ResolveDocumentOutput,
};
pub use session::{ApplyCommandRequest, VersionLensSession, version_lens_session};
pub use versionlens_suggestions::{Suggestion, SuggestionStatus};
pub(crate) fn default<T: Default>() -> T {
    <T as Default>::default()
}

pub(crate) fn parse_semver(value: &str) -> Result<SemverVersion, SemverError> {
    value.parse()
}

pub(crate) fn recover_poison<T>(poisoned: SyncPoisonError<T>) -> T {
    poisoned.into_inner()
}

pub(crate) const fn duration_from_millis(milliseconds: u64) -> StdDuration {
    std::time::Duration::from_millis(milliseconds)
}

pub(crate) fn path(value: &str) -> &StdPath {
    value.as_ref()
}

pub(crate) fn mutex<T>(value: T) -> StdMutex<T> {
    std::sync::Mutex::new(value)
}

pub(crate) fn boxed<T>(value: T) -> Box<T> {
    <Box<_>>::new(value)
}

pub(crate) fn arc<T>(value: T) -> StdArc<T> {
    std::sync::Arc::new(value)
}

pub(crate) fn memory_cache<T>(ttl: StdDuration) -> VersionLensMemoryCache<T> {
    <VersionLensMemoryCache<T>>::new(ttl)
}

pub(crate) fn anyhow_error<E>(error: E) -> AnyhowError
where
    E: StdError + Send + Sync + 'static,
{
    error.into()
}

pub(crate) fn clone_arc<T>(value: &StdArc<T>) -> StdArc<T> {
    value.clone()
}

#[cfg(test)]
pub(crate) fn session_config_from_input(input: SessionConfigInput) -> SessionConfig {
    config::session_config_from_input(input)
}

#[cfg(test)]
pub(crate) fn system_time_now() -> StdSystemTime {
    std::time::SystemTime::now()
}

#[cfg(test)]
pub(crate) fn tcp_listener_bind(addr: &str) -> IoResult<StdTcpListener> {
    std::net::TcpListener::bind(addr)
}

pub(crate) fn string_from_utf8(value: Vec<u8>) -> Result<String, StringFromUtf8Error> {
    value.try_into()
}

pub(crate) fn parse_semver_req(value: &str) -> Result<SemverVersionReq, SemverError> {
    value.parse()
}
