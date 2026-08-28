use self::collect::{collect_catalog, collect_catalogs, collect_package_extensions};
use crate::support;
use marked_yaml::types::MarkedMappingNode;

use versionlens_model::Dependency;

mod collect;
mod dependency;
mod nodes;
mod paths;

use paths::{root_dependency_group, selected_dependency_paths};

struct PnpmCollectContext<'a> {
    text: &'a str,
    dependency_paths: &'a [&'a str],
}

pub(crate) fn parse_pnpm_yaml_with_paths(text: &str, dependency_paths: &[&str]) -> Vec<Dependency> {
    support::with_yaml_mapping(text, |root| {
        let mut dependencies = vec![];
        let dependency_paths = selected_dependency_paths(dependency_paths);
        let context = PnpmCollectContext {
            text,
            dependency_paths: &dependency_paths,
        };
        collect_catalog(&context, root, "catalog", &mut dependencies);
        collect_catalog(&context, root, "overrides", &mut dependencies);
        collect_configured_root_groups(&context, root, &mut dependencies);
        collect_catalogs(&context, root, &mut dependencies);
        collect_package_extensions(&context, root, &mut dependencies);
        dependencies
    })
    .unwrap_or_default()
}

fn collect_configured_root_groups(
    context: &PnpmCollectContext<'_>,
    root: &MarkedMappingNode,
    out: &mut Vec<Dependency>,
) {
    for path in context.dependency_paths {
        let Some(group) = root_dependency_group(path) else {
            continue;
        };
        collect_catalog(context, root, group, out);
    }
}

#[cfg(test)]
mod tests;
