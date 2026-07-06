use versionlens_parsers::Dependency;

use crate::model::Suggestion;
use crate::resolve::resolve_dependency;

type Dependencies = Vec<Dependency>;
type Suggestions = Vec<Suggestion>;

pub fn unresolved(dependencies: Dependencies) -> Suggestions {
    dependencies
        .into_iter()
        .map(|dependency| resolve_dependency(dependency, None))
        .collect()
}

pub fn resolve_with_latest(dependencies: Dependencies, latest: &str) -> Suggestions {
    dependencies
        .into_iter()
        .map(|dependency| resolve_dependency(dependency, Some(latest.to_owned())))
        .collect()
}

#[cfg(test)]
mod tests;
