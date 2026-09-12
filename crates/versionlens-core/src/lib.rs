mod cache;
mod command;
mod concurrency;
mod config;
mod contract;
mod dependency;
mod docker;
mod dotnet_sources;
mod error;
mod fetch;
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
mod support;
mod vulnerability;
mod workspace;

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
pub use contract::validate_workspace_edit_plan;
pub use contract::{
    AnalyzeDocumentOutput, AuthorizationRequestPayload, RegistryResponseInput,
    ResolveDocumentOutput,
};
pub use dependency::dependency_payload;
pub use dotnet_sources::dotnet_registry_source_urls;
pub use session::{
    ApplyCommandRequest, SessionTask, TaskCancellation, VersionLensSession, WorkspaceCheckEvent,
    WorkspaceCheckResult, WorkspaceChecking, WorkspaceCheckingOptions, WorkspaceDiscovery,
    WorkspaceDiscoveryCancellation, WorkspaceDiscoveryFailure, WorkspaceDiscoveryFailureKind,
    WorkspaceDiscoveryFileSize, WorkspaceDiscoveryIoOperation, WorkspaceDiscoveryLimits,
    WorkspaceDiscoveryOptions, WorkspaceProviderExclusion, version_lens_session,
    workspace_exclusion_matches, workspace_exclusion_path,
};
pub(crate) use support::{
    anyhow_error, arc, boxed, default, duration_from_millis, memory_cache, mutex, parse_semver,
    parse_semver_req, path, recover_poison, string_from_utf8,
};
pub use versionlens_suggestions::{Suggestion, SuggestionStatus};

pub use workspace::{workspace_file_uri, workspace_path};
