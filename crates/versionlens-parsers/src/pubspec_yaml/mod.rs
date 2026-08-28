use self::collect::collect_pubspec_workspace;
use crate::support;

use versionlens_model::Dependency;

mod collect;
mod dependency;
mod paths;

use collect::{PubspecCollectContext, collect_pubspec_dependency_groups, collect_pubspec_version};
use paths::selected_dependency_paths;

pub(crate) fn parse_pubspec_yaml_with_paths(
    text: &str,
    dependency_paths: &[&str],
) -> Vec<Dependency> {
    support::with_yaml_mapping(text, |root| {
        let mut dependencies = vec![];
        let dependency_paths = selected_dependency_paths(dependency_paths);
        let context = PubspecCollectContext {
            text,
            dependency_paths: &dependency_paths,
        };
        collect_pubspec_version(&context, root, &mut dependencies);
        collect_pubspec_workspace(&context, root, &mut dependencies);
        collect_pubspec_dependency_groups(&context, root, &mut dependencies);
        dependencies
    })
    .unwrap_or_default()
}

#[cfg(test)]
mod tests;
