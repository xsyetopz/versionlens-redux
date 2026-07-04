use versionlens_parsers::Dependency;
use versionlens_suggestions::{Suggestion, resolve_dependency};

use crate::VersionLensSession;
use crate::cache::cache_key;
use crate::non_registry::{deno_import_has_no_suggestions, known_non_registry_suggestion};
use crate::project::project_version_latest;

impl VersionLensSession {
    pub(crate) fn cached_suggestion(&self, dependency: &Dependency) -> Option<Suggestion> {
        if let Some(suggestion) = self.cached_resolved_suggestion(dependency) {
            return Some(suggestion);
        }

        if let Some(latest) = project_version_latest(dependency, None) {
            return Some(resolve_dependency(dependency.to_owned(), Some(latest)));
        }

        if deno_import_has_no_suggestions(dependency) {
            return None;
        }

        let dependency = match known_non_registry_suggestion(dependency.to_owned(), None) {
            Ok(suggestion) => return Some(suggestion),
            Err(dependency) => *dependency,
        };
        let latest = self.cached_latest(&cache_key(dependency.ecosystem, &dependency.name))?;

        Some(resolve_dependency(dependency, Some(latest)))
    }
}
