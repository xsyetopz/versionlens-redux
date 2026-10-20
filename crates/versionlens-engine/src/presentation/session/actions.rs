use std::collections::HashSet;

use versionlens_model::{DocumentInput, Range};

use crate::presentation::view::BumpActionView;
use crate::{Suggestion, SuggestionStatus, VersionLensSession};

impl VersionLensSession {
    /// The bump quick fixes for the requirements that `range` touches. They
    /// read only cached results, so they never wait for a registry, and a
    /// document that was not resolved yet gets none.
    pub fn bump_actions(&self, input: &DocumentInput, range: Range) -> Vec<BumpActionView> {
        self.synchronize_persistent_cache();
        let manifest_kind = self.classify_document(input);
        let context = self.registry_context(input, manifest_kind);
        let scope = self.document_cache_scope(&context, input);
        self.dependencies(input)
            .iter()
            .filter(|dependency| overlaps(dependency.requirement_range, range))
            .filter_map(|dependency| self.cached_suggestion(dependency, &scope))
            .flat_map(|suggestion| self.suggestion_actions(&suggestion))
            .collect()
    }

    /// One action per distinct bump text, in the order the lenses offer them,
    /// then latest when no choice names it.
    fn suggestion_actions(&self, suggestion: &Suggestion) -> Vec<BumpActionView> {
        if !matches!(
            suggestion.status,
            SuggestionStatus::UpdateAvailable
                | SuggestionStatus::Satisfies
                | SuggestionStatus::SatisfiesLatest
                | SuggestionStatus::InvalidRange
        ) {
            return vec![];
        }
        let latest = suggestion.latest.as_deref();
        let mut versions = suggestion
            .choices
            .iter()
            .map(|choice| choice.version.as_str())
            .collect::<Vec<_>>();
        if let Some(latest) = latest.filter(|latest| !versions.contains(latest)) {
            versions.push(latest);
        }
        let diagnostic = self.update_diagnostic(suggestion);
        let current = suggestion.dependency.requirement.trim();
        let mut seen = HashSet::new();
        versions
            .into_iter()
            .filter_map(|version| {
                let mut target = suggestion.clone();
                target.latest = Some(version.to_owned());
                if target.status == SuggestionStatus::SatisfiesLatest {
                    target.status = SuggestionStatus::UpdateAvailable;
                }
                let edit = versionlens_update::update_edit(&target)?;
                (edit.new_text != current && seen.insert(edit.new_text.clone())).then(|| {
                    BumpActionView {
                        title: format!("Bump to {}", edit.new_text),
                        preferred: Some(version) == latest,
                        diagnostic: diagnostic.clone(),
                        edit,
                    }
                })
            })
            .collect()
    }
}

/// Whether two ranges share a position. An empty range, such as a cursor,
/// counts at either end of a requirement.
fn overlaps(requirement: Range, range: Range) -> bool {
    let key = |position: versionlens_model::Position| (position.line, position.character);
    key(range.start) <= key(requirement.end) && key(requirement.start) <= key(range.end)
}
