use versionlens_model::Dependency;
use versionlens_update::{Suggestion, resolve_dependency};

use super::storage::CacheScope;
use crate::VersionLensSession;
use crate::non_registry::{deno_import_has_no_suggestions, known_non_registry_suggestion};
use crate::project::project_version_latest;

impl VersionLensSession {
    pub(crate) fn cached_suggestion(
        &self,
        dependency: &Dependency,
        scope: &CacheScope,
    ) -> Option<Suggestion> {
        if let Some(suggestion) = self.cached_resolved_suggestion(dependency, scope) {
            return Some(suggestion);
        }

        if let Some(latest) = project_version_latest(dependency, None) {
            return Some(resolve_dependency(dependency.to_owned(), Some(latest)));
        }

        if deno_import_has_no_suggestions(dependency) {
            return None;
        }

        known_non_registry_suggestion(dependency.to_owned(), None).ok()
    }
}
