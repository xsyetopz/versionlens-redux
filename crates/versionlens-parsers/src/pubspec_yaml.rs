use marked_yaml::parse_yaml;

use crate::model::Dependency;

mod collect;
mod dependency;
mod paths;

use collect::{PubspecCollectContext, collect_pubspec_dependency_groups, collect_pubspec_version};
use paths::selected_dependency_paths;

pub(crate) fn parse_pubspec_yaml_with_paths(
    text: &str,
    dependency_paths: &[&str],
) -> Vec<Dependency> {
    let Ok(root) = parse_yaml(0, text) else {
        return Vec::new();
    };
    let Some(root) = root.as_mapping() else {
        return Vec::new();
    };

    let mut dependencies = Vec::new();
    let dependency_paths = selected_dependency_paths(dependency_paths);
    let context = PubspecCollectContext {
        text,
        dependency_paths: &dependency_paths,
    };
    collect_pubspec_version(&context, root, &mut dependencies);
    collect_pubspec_dependency_groups(&context, root, &mut dependencies);

    dependencies
}

#[cfg(test)]
mod tests;
