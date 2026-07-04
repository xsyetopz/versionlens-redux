use versionlens_parsers::Dependency;
use versionlens_suggestions::UpdateChoice;

use crate::VersionLensSession;
use crate::cache::latest_cache_key;
use crate::model::RegistryResponseInput;
use crate::prerelease::{npm_requirement_may_be_dist_tag, requirement_mentions_prerelease};
use crate::registry::RegistryContext;

use super::LatestLookup;

impl VersionLensSession {
    pub(in crate::session::resolution::latest) fn resolve_cacheable_latest(
        &self,
        dependency: &Dependency,
        responses: &[RegistryResponseInput],
        has_registry_response: bool,
        context: &RegistryContext,
    ) -> LatestLookup {
        let key = latest_cache_key(dependency);
        if !has_registry_response && let Some(cached) = self.cache().get(&key) {
            return cached_latest_lookup(cached);
        }

        match self.lookup_latest(dependency, responses, has_registry_response, context) {
            Ok(lookup) => {
                if let Some(latest) = &lookup.latest {
                    self.cache().insert_with_ttl(
                        key,
                        crate::session::cache::CachedLatest {
                            latest: latest.to_owned(),
                            builds: copied_strings(&lookup.builds),
                            choices: copied_update_choices(&lookup.choices),
                        },
                        self.cache_ttl(dependency.ecosystem, context.manifest_kind()),
                    );
                }
                lookup
            }
            Err(fetch_error) => LatestLookup {
                latest: None,
                builds: Vec::new(),
                choices: Vec::new(),
                fetch_error: Some(fetch_error),
            },
        }
    }

    pub(in crate::session::resolution::latest) fn resolve_uncached_latest(
        &self,
        dependency: &Dependency,
        responses: &[RegistryResponseInput],
        has_registry_response: bool,
        context: &RegistryContext,
    ) -> LatestLookup {
        match self.lookup_latest(dependency, responses, has_registry_response, context) {
            Ok(lookup) => lookup,
            Err(fetch_error) => LatestLookup {
                latest: None,
                builds: Vec::new(),
                choices: Vec::new(),
                fetch_error: Some(fetch_error),
            },
        }
    }

    pub(in crate::session::resolution::latest) fn uses_shared_latest_cache(
        &self,
        dependency: &Dependency,
        context: &RegistryContext,
    ) -> bool {
        !context.has_urls()
            && !npm_requirement_may_be_dist_tag(dependency)
            && (self.config.show_prereleases
                || !requirement_mentions_prerelease(&dependency.requirement))
    }
}

fn cached_latest_lookup(cached: &crate::session::cache::CachedLatest) -> LatestLookup {
    LatestLookup {
        latest: Some(String::from(cached.latest.as_str())),
        builds: copied_strings(&cached.builds),
        choices: copied_update_choices(&cached.choices),
        fetch_error: None,
    }
}

fn copied_strings(values: &[String]) -> Vec<String> {
    values
        .iter()
        .map(|value| String::from(value.as_str()))
        .collect()
}

fn copied_update_choices(choices: &[UpdateChoice]) -> Vec<UpdateChoice> {
    choices
        .iter()
        .map(|choice| UpdateChoice {
            label: String::from(choice.label.as_str()),
            version: String::from(choice.version.as_str()),
            command: String::from(choice.command.as_str()),
        })
        .collect()
}
