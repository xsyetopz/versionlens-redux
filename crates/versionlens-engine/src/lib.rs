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
#[cfg(test)]
mod test_support;
mod vulnerability;
mod workspace;

pub use config::{SessionConfig, Settings, UpdateSeverity};
pub use contract::validate_workspace_edit_plan;
pub use contract::{AnalyzeDocumentOutput, RegistryResponseInput, ResolveDocumentOutput};
pub use presentation::view::{
    Action, BumpActionView, DependencyView, DiagnosticView, LensView, Severity, UPDATE_AVAILABLE,
    UpdateMode,
};
pub use session::{
    AuthorizationFuture, AuthorizationSource, CHECKING_CANCELLED, SessionTask, VersionLensSession,
    WorkspaceCheckEvent, WorkspaceCheckResult, WorkspaceChecking, WorkspaceCheckingOptions,
    WorkspaceDiscoveryFailure, WorkspaceDiscoveryFailureKind, WorkspaceDiscoveryOptions,
    version_lens_session, workspace_exclusion_matches, workspace_exclusion_path,
};
pub use versionlens_fetch::{Credential, application_cache_directory};
pub use workspace::{workspace_file_uri, workspace_path};

pub(crate) use config::{
    DependencyPropertyConfig, ProviderCacheConfig, RegistryUrlConfig, SuggestionIndicators,
};
pub(crate) use session::WorkspaceDiscovery;
pub(crate) use versionlens_model::parse_semver;
pub(crate) use versionlens_update::{Suggestion, SuggestionStatus};

#[cfg(test)]
pub(crate) use config::{
    EnabledProviderConfig, FilePatternConfig, PrereleaseTagConfig, ProviderHttpConfig,
    ProviderSettings, enabled_provider_config_from_name, standard_suggestion_indicators,
};
#[cfg(test)]
pub(crate) use presentation::view::SuggestionView;
#[cfg(test)]
pub(crate) use session::{ApplyCommandRequest, WorkspaceDiscoveryFileSize};
