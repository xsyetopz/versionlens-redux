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
mod view;
mod vulnerability;
mod workspace;

pub use config::{SessionConfig, Settings, UpdateSeverity};
pub use contract::validate_workspace_edit_plan;
pub use contract::{AnalyzeDocumentOutput, RegistryResponseInput, ResolveDocumentOutput};
pub use session::{
    AuthorizationFuture, AuthorizationSource, CHECKING_CANCELLED, SessionTask, VersionLensSession,
    WorkspaceCheckEvent, WorkspaceCheckResult, WorkspaceChecking, WorkspaceCheckingOptions,
    WorkspaceDiscoveryFailure, WorkspaceDiscoveryFailureKind, WorkspaceDiscoveryOptions,
    version_lens_session, workspace_exclusion_matches, workspace_exclusion_path,
};
pub use versionlens_fetch::{Credential, application_cache_directory};
pub use view::{
    Action, BumpActionView, DependencyView, DiagnosticView, LensView, Severity, UPDATE_AVAILABLE,
    UpdateMode,
};
pub use workspace::{workspace_file_uri, workspace_path};

pub(crate) use config::{
    DependencyPropertyConfig, ProviderCacheConfig, RegistryUrlConfig, SuggestionIndicators,
};
pub(crate) use session::WorkspaceDiscovery;
pub(crate) use support::{
    anyhow_error, boxed, default, duration_from_millis, memory_cache, mutex, path, recover_poison,
    string_from_utf8,
};
pub(crate) use versionlens_model::parse_semver;
pub(crate) use versionlens_update::{Suggestion, SuggestionStatus};

#[cfg(test)]
pub(crate) use config::{
    EnabledProviderConfig, FilePatternConfig, PrereleaseTagConfig, ProviderHttpConfig,
    ProviderSettings, enabled_provider_config_from_name, standard_suggestion_indicators,
};
#[cfg(test)]
pub(crate) use session::{ApplyCommandRequest, WorkspaceDiscoveryFileSize};
#[cfg(test)]
pub(crate) use view::SuggestionView;
