use self::collect::{collect_catalog, collect_catalogs, collect_package_extensions};
use crate::support;
use marked_yaml::types::MarkedMappingNode;

use versionlens_model::Dependency;

mod collect;
mod dependency;

struct PnpmCollectContext<'a> {
    text: &'a str,
    dependency_paths: &'a [&'a str],
}

pub(crate) fn parse_pnpm_yaml_with_paths(text: &str, dependency_paths: &[&str]) -> Vec<Dependency> {
    support::with_yaml_mapping(text, |root| {
        let mut dependencies = vec![];
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

fn root_dependency_group(path: &str) -> Option<&str> {
    if path.split_once('.').is_some() || matches!(path, "catalog" | "overrides") {
        None
    } else {
        Some(path)
    }
}

#[cfg(test)]
mod tests {
use crate::document::test_support::{extract_range, parse_fixture};
use versionlens_model::Ecosystem::Npm;
use versionlens_model::VersionableKind;

#[test]
fn parses_catalog_and_named_catalogs() {
    let text = package_file_fixture("parses-catalog-and-named-catalogs.txt");
    let dependencies = parse_fixture(text, "file:///work/pnpm-workspace.yaml", "yaml");

    assert_eq!(dependencies.len(), 3);
    crate::support::tests::assert_dependency(
        &dependencies,
        crate::support::tests::DependencyExpectation::new(0, Npm, "catalog", "react", "^16.14.0"),
    );
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "^16.14.0"
    );
    assert_eq!(dependencies[1].group, "catalogs.react17");
    assert_eq!(dependencies[1].name, "react");
    assert_eq!(dependencies[1].requirement, "^17.0.2");
    assert_eq!(
        extract_range(text, dependencies[1].requirement_range),
        "^17.0.2"
    );
    assert_eq!(dependencies[2].name, "react-dom");
}

#[test]
fn parses_yarnrc_catalog() {
    let text = package_file_fixture("parses-yarnrc-catalog.txt");
    let dependencies = parse_fixture(text, "file:///work/.yarnrc.yaml", "yaml");

    assert_eq!(dependencies.len(), 1);
    crate::support::tests::assert_dependency(
        &dependencies,
        crate::support::tests::DependencyExpectation::new(0, Npm, "catalog", "react", "^18.2.0"),
    );
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "^18.2.0"
    );
}

#[test]
fn parses_smoke_yarnrc_smoke_shapes() {
    let text = package_file_fixture("parses-smoke-yarnrc-smoke-shapes.yarnrc.yaml");
    let dependencies = parse_fixture(text, "file:///work/.yarnrc.yml", "yaml");

    assert_eq!(dependencies.len(), 5);
    crate::support::tests::assert_dependency(
        &dependencies,
        crate::support::tests::DependencyExpectation::new(0, Npm, "catalog", "lodash", "^4.18.1"),
    );
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "^4.18.1"
    );
    assert_eq!(dependencies[1].group, "catalogs.react18");
    assert_eq!(dependencies[1].name, "react");
    assert_eq!(dependencies[2].name, "react-dom");
    assert_eq!(dependencies[3].group, "catalogs.react17");
    assert_eq!(dependencies[3].name, "react");
    assert_eq!(dependencies[4].name, "react-dom");
}

#[test]
fn parses_root_overrides() {
    let text = package_file_fixture("parses-root-overrides.txt");
    let dependencies = parse_fixture(text, "file:///work/pnpm-workspace.yaml", "yaml");

    assert_eq!(dependencies.len(), 1);
    crate::support::tests::assert_dependency(
        &dependencies,
        crate::support::tests::DependencyExpectation::new(0, Npm, "overrides", "vite", "^5.4.0"),
    );
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "^5.4.0"
    );
}

#[test]
fn parses_package_extensions() {
    let text = package_file_fixture("parses-package-extensions.yaml");
    let dependencies = parse_fixture(text, "file:///work/pnpm-workspace.yaml", "yaml");

    assert_eq!(dependencies.len(), 3);
    assert_eq!(dependencies[0].ecosystem, Npm);
    assert_eq!(
        dependencies[0].group,
        "packageExtensions.react@18.dependencies"
    );
    assert_eq!(dependencies[0].name, "scheduler");
    assert_eq!(dependencies[0].requirement, "^0.23.0");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "^0.23.0"
    );
    assert_eq!(
        dependencies[1].group,
        "packageExtensions.react@18.peerDependencies"
    );
    assert_eq!(dependencies[1].name, "@types/react");
    assert_eq!(dependencies[1].requirement, "^18.0.0");
    assert_eq!(
        dependencies[2].group,
        "packageExtensions.vite@5.optionalDependencies"
    );
    assert_eq!(dependencies[2].name, "fsevents");
}

#[test]
fn parses_smoke_pnpm_workspace_smoke_shapes() {
    let text = package_file_fixture("parses-smoke-pnpm-workspace-smoke-shapes.txt");
    let dependencies = parse_fixture(text, "file:///work/pnpm-workspace.yaml", "yaml");

    assert_eq!(dependencies.len(), 6);
    crate::support::tests::assert_dependency(
        &dependencies,
        crate::support::tests::DependencyExpectation::new(0, Npm, "catalog", "react", "^19.2.7"),
    );
    assert_eq!(dependencies[1].name, "react-dom");
    assert_eq!(dependencies[1].requirement, "^19.2.7");
    assert_eq!(dependencies[2].group, "overrides");
    assert_eq!(dependencies[2].name, "typescript");
    assert_eq!(dependencies[2].requirement, "^6.0.3");
    assert_eq!(dependencies[3].group, "catalogs.react18");
    assert_eq!(dependencies[3].name, "react");
    assert_eq!(dependencies[3].requirement, "^18.3.1");
    assert_eq!(dependencies[4].name, "react-dom");
    assert_eq!(
        dependencies[5].group,
        "packageExtensions.react-redux.peerDependencies"
    );
    assert_eq!(dependencies[5].name, "react-dom");
    assert_eq!(dependencies[5].requirement, "*");
    assert_eq!(extract_range(text, dependencies[5].requirement_range), "*");
}

#[test]
fn parses_package_yaml_npm_aliases_and_surfaces_workspace_catalog_specs() {
    let text = package_file_fixture(
        "parses-package-yaml-npm-aliases-and-skips-workspace-catalog-specs.txt",
    );
    let dependencies = parse_fixture(text, "file:///work/package.yaml", "yaml");

    assert_eq!(dependencies.len(), 4);
    crate::support::tests::assert_dependency(
        &dependencies,
        crate::support::tests::DependencyExpectation::new(
            0,
            Npm,
            "dependencies",
            "typescript",
            "^6.0.3",
        ),
    );
    assert_eq!(dependencies[0].requirement_prefix, "npm:typescript@");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "npm:typescript@^6.0.3"
    );
    assert_eq!(dependencies[1].name, "types-react");
    assert_eq!(dependencies[1].requirement, "");
    assert_eq!(dependencies[1].requirement_prefix, "npm:types-react@");
    assert_eq!(extract_range(text, dependencies[1].requirement_range), "");
    assert_eq!(dependencies[2].name, "workspace-only");
    assert_eq!(dependencies[2].requirement, "workspace:*");
    assert_eq!(
        extract_range(text, dependencies[2].requirement_range),
        "workspace:*"
    );
    assert_eq!(
        dependencies[2].versionable_kind(),
        VersionableKind::WorkspaceReference
    );
    assert_eq!(dependencies[2].hosted_url.as_deref(), Some("workspace"));
    assert_eq!(dependencies[3].name, "catalog-only");
    assert_eq!(dependencies[3].requirement, "catalog:");
    assert_eq!(
        extract_range(text, dependencies[3].requirement_range),
        "catalog:"
    );
    assert_eq!(
        dependencies[3].versionable_kind(),
        VersionableKind::WorkspaceReference
    );
    assert_eq!(dependencies[3].hosted_url.as_deref(), Some("workspace"));
}

#[test]
fn parses_package_yaml_dependency_groups() {
    let text = package_file_fixture("parses-package-yaml-dependency-groups.txt");
    let dependencies = parse_fixture(text, "file:///work/package.yaml", "yaml");

    assert_eq!(dependencies.len(), 4);
    assert_eq!(dependencies[0].name, "react");
    assert_eq!(dependencies[0].requirement, "^19.0.0");
    assert_eq!(dependencies[0].ecosystem, Npm);
    assert_eq!(dependencies[0].group, "dependencies");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "^19.0.0"
    );
    assert_eq!(dependencies[1].group, "devDependencies");
    assert_eq!(dependencies[1].name, "typescript");
    assert_eq!(dependencies[2].group, "peerDependencies");
    assert_eq!(dependencies[2].name, "@types/react");
    assert_eq!(dependencies[3].group, "optionalDependencies");
    assert_eq!(dependencies[3].name, "fsevents");
}

fn package_file_fixture(name: &str) -> &'static str {
    crate::support::tests::fixture(
        "tests/fixtures/versionlens-parsers/src/pnpm_yaml/tests",
        name,
    )
}
}
