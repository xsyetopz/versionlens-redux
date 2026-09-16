use versionlens_providers::RuntimeSource;
use versionlens_suggestions::{Suggestion, SuggestionStatus, error, resolve_dependency};

use super::latest::LatestResolutionRequest;
use crate::VersionLensSession;

impl VersionLensSession {
    pub(super) async fn runtime_suggestion(
        &self,
        request: LatestResolutionRequest<'_>,
    ) -> Suggestion {
        let dependency = request.dependency.clone();
        let source = match RuntimeSource::for_dependency(&dependency) {
            Ok(source) => source,
            Err(message) => return error(dependency, message),
        };
        let lookup = self.resolve_latest(request).await;
        if let Some(error_message) = lookup.fetch_error {
            return error(dependency, error_message.to_string());
        }
        let Some(latest) = lookup.latest else {
            return error(
                dependency,
                "no published runtime release was found".to_owned(),
            );
        };
        if dependency.is_runtime_constraint()
            || source.is_constraint(&dependency.requirement)
            || source.is_channel(&dependency.requirement)
        {
            return Suggestion {
                status: if dependency.is_runtime_constraint()
                    || source.is_constraint(&dependency.requirement)
                {
                    SuggestionStatus::SatisfiesLatest
                } else {
                    SuggestionStatus::Current
                },
                dependency,
                latest: Some(latest),
                resolved: None,
                builds: vec![],
                choices: vec![],
            };
        }
        let mut comparable = dependency.clone();
        if let Some((version, _)) = dependency.requirement.split_once("+sha") {
            comparable.requirement = version.to_owned();
        }
        let mut suggestion = resolve_dependency(comparable, Some(latest));
        suggestion.dependency = dependency;
        suggestion.choices = lookup.choices;
        suggestion
    }
}
