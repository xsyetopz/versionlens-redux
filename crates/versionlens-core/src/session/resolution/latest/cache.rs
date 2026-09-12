use versionlens_suggestions::{UpdateChoice, deduplicate_update_choices};

use crate::VersionLensSession;
use crate::cache::suggestion_cache_key;

use super::{LatestLookup, LatestResolutionRequest};
use crate::session::cache::CachedLatest;

impl VersionLensSession {
    pub(in crate::session::resolution::latest) fn resolve_cacheable_latest(
        &self,
        request: LatestResolutionRequest<'_>,
    ) -> LatestLookup {
        let LatestResolutionRequest {
            dependency,
            responses,
            has_registry_response,
            context,
            operation,
        } = request;
        let key = suggestion_cache_key(dependency, &self.cache_scope(context));
        let persistent_key = self.persistent_latest_key(dependency, context);
        if !has_registry_response
            && let Some(cached) = self.cache().get(&key)
            && let Some(lookup) = cached_latest_lookup(dependency, cached)
        {
            return lookup;
        }

        if !has_registry_response
            && let Some(persistent_key) = persistent_key.as_deref()
            && let Some((cached, ttl)) = self.load_persistent_latest(persistent_key)
            && let Some(lookup) = cached_latest_lookup(dependency, &cached)
        {
            let mut cache = self.cache();
            if operation.is_current() {
                cache.insert_with_ttl(key, cached, ttl);
            }
            drop(cache);
            return lookup;
        }

        match self.lookup_latest(request) {
            Ok(mut lookup) => {
                deduplicate_update_choices(&mut lookup.choices);
                lookup.fixed_requirement_matched =
                    super::super::dependency::fixed_requirement_matches_response(
                        dependency, responses,
                    );
                if let Some(latest) = &lookup.latest {
                    let mut cache = self.cache();
                    if !operation.is_current() {
                        return lookup;
                    }
                    let cached = CachedLatest {
                        latest: latest.to_owned(),
                        builds: copied_strings(&lookup.builds),
                        choices: copied_update_choices(&lookup.choices),
                        fixed_requirement_matched: Some(lookup.fixed_requirement_matched),
                    };
                    let ttl = self.cache_ttl(dependency.ecosystem, context.manifest_kind());
                    cache.insert_with_ttl(key, cached.clone(), ttl);
                    drop(cache);
                    if let Some(persistent_key) = persistent_key {
                        self.store_persistent_latest(persistent_key, &cached, ttl, operation);
                    }
                }
                lookup
            }
            Err(fetch_error) => failed_latest_lookup(fetch_error),
        }
    }
}

fn failed_latest_lookup(fetch_error: crate::error::FetchError) -> LatestLookup {
    LatestLookup {
        latest: None,
        builds: vec![],
        choices: vec![],
        fetch_error: Some(fetch_error),
        fixed_requirement_matched: false,
    }
}

fn cached_latest_lookup(
    dependency: &versionlens_model::Dependency,
    cached: &CachedLatest,
) -> Option<LatestLookup> {
    if cached.fixed_requirement_matched.is_none()
        && super::super::dependency::fixed_requirement_requires_response_proof(dependency)
    {
        return None;
    }

    let mut choices = copied_update_choices(&cached.choices);
    deduplicate_update_choices(&mut choices);
    Some(LatestLookup {
        latest: Some(cached.latest.as_str().to_owned()),
        builds: copied_strings(&cached.builds),
        choices,
        fetch_error: None,
        fixed_requirement_matched: cached.fixed_requirement_matched.unwrap_or(false),
    })
}

fn copied_strings(values: &[String]) -> Vec<String> {
    values
        .iter()
        .map(|value| value.as_str().to_owned())
        .collect()
}

fn copied_update_choices(choices: &[UpdateChoice]) -> Vec<UpdateChoice> {
    choices
        .iter()
        .map(|choice| UpdateChoice {
            label: choice.label.as_str().to_owned(),
            version: choice.version.as_str().to_owned(),
            command: choice.command.as_str().to_owned(),
            replacement: choice.replacement.as_deref().map(str::to_owned),
        })
        .collect()
}
