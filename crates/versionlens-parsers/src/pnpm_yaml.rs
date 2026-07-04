use marked_yaml::parse_yaml;

use crate::model::Dependency;

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
    let Ok(root) = parse_yaml(0, text) else {
        return Vec::new();
    };
    let Some(root) = root.as_mapping() else {
        return Vec::new();
    };

    let mut dependencies = Vec::new();
    let dependency_paths = selected_dependency_paths(dependency_paths);
    let context = PnpmCollectContext {
        text,
        dependency_paths: &dependency_paths,
    };
    collect::collect_catalog(&context, root, "catalog", &mut dependencies);
    collect::collect_catalog(&context, root, "overrides", &mut dependencies);
    collect_configured_root_groups(&context, root, &mut dependencies);
    collect::collect_catalogs(&context, root, &mut dependencies);
    collect::collect_package_extensions(&context, root, &mut dependencies);
    dependencies
}

fn collect_configured_root_groups(
    context: &PnpmCollectContext<'_>,
    root: &marked_yaml::types::MarkedMappingNode,
    out: &mut Vec<Dependency>,
) {
    for path in context.dependency_paths {
        let Some(group) = root_dependency_group(path) else {
            continue;
        };
        collect::collect_catalog(context, root, group, out);
    }
}

#[cfg(test)]
mod tests;
