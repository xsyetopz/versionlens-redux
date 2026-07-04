use versionlens_parsers::Dependency;

use crate::model::Suggestion;
use crate::resolve::resolve_dependency;

pub fn unresolved(dependencies: Vec<Dependency>) -> Vec<Suggestion> {
    dependencies
        .into_iter()
        .map(|dependency| resolve_dependency(dependency, None))
        .collect()
}

pub fn resolve_with_latest(dependencies: Vec<Dependency>, latest: &str) -> Vec<Suggestion> {
    dependencies
        .into_iter()
        .map(|dependency| resolve_dependency(dependency, Some(latest.to_owned())))
        .collect()
}

#[cfg(test)]
mod tests;
