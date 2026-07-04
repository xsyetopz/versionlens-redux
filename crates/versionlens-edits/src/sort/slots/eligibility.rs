use std::collections::{BTreeMap, BTreeSet};

use versionlens_parsers::{Dependency, Ecosystem};

use super::lines::{dependency_end_line, dependency_start_line};

const CARGO_DEPENDENCY_GROUPS: &[&str] =
    &["dependencies", "dev-dependencies", "build-dependencies"];
const NPM_DEPENDENCY_GROUPS: &[&str] = &[
    "dependencies",
    "devDependencies",
    "peerDependencies",
    "optionalDependencies",
    "bundledDependencies",
    "bundleDependencies",
];
const PUB_DEPENDENCY_GROUPS: &[&str] =
    &["dependencies", "dev_dependencies", "dependency_overrides"];

pub fn can_sort_dependencies(dependencies: &[Dependency]) -> bool {
    let sortable = sortable_dependencies(dependencies);

    has_sortable_group(sortable.iter().copied())
        && dependency_lines_are_unique(sortable.iter().copied())
        && sortable
            .iter()
            .all(|dependency| has_sortable_span(dependency))
}

pub(in crate::sort) fn is_sortable_dependency(dependency: &Dependency) -> bool {
    let group = dependency.group.as_str();

    match dependency.ecosystem {
        Ecosystem::Cargo => is_cargo_sortable_group(group),
        Ecosystem::Composer => matches!(group, "require" | "require-dev"),
        Ecosystem::Deno => is_deno_sortable_group(group),
        Ecosystem::Dotnet => is_dotnet_sortable_dependency(dependency),
        Ecosystem::Dub => matches!(group, "dependencies" | "versions"),
        Ecosystem::Go => matches!(group, "require" | "exclude"),
        Ecosystem::Maven => is_maven_sortable_group(group),
        Ecosystem::Npm => is_npm_sortable_group(group),
        Ecosystem::Python => is_python_sortable_group(group),
        Ecosystem::Pub => PUB_DEPENDENCY_GROUPS.contains(&group),
        Ecosystem::Ruby => true,
        Ecosystem::Docker => false,
    }
}

fn is_python_sortable_group(group: &str) -> bool {
    matches!(
        group,
        "dependencies"
            | "requirements"
            | "packages"
            | "dev-packages"
            | "project.dependencies"
            | "tool.poetry.dependencies"
            | "tool.poetry.dev-dependencies"
    ) || group.starts_with("project.optional-dependencies.")
        || group.starts_with("dependency-groups.")
        || group.starts_with("tool.poetry.dependencies.")
        || group.starts_with("tool.poetry.dev-dependencies.")
        || is_poetry_named_group_dependency(group)
}

fn is_poetry_named_group_dependency(group: &str) -> bool {
    let Some(named_group) = group.strip_prefix("tool.poetry.group.") else {
        return false;
    };

    named_group.ends_with(".dependencies") || named_group.contains(".dependencies.")
}

fn is_dotnet_sortable_dependency(dependency: &Dependency) -> bool {
    matches!(
        dependency.group.as_str(),
        "PackageReference" | "PackageVersion" | "GlobalPackageReference" | "DotNetCliToolReference"
    ) && dependency.requirement_prefix.is_empty()
        && dependency.requirement_suffix.is_empty()
        && range_contains_requirement(dependency)
}

fn range_contains_requirement(dependency: &Dependency) -> bool {
    let starts_before_requirement = dependency.range.start.line
        < dependency.requirement_range.start.line
        || (dependency.range.start.line == dependency.requirement_range.start.line
            && dependency.range.start.character <= dependency.requirement_range.start.character);
    let ends_after_requirement = dependency.range.end.line > dependency.requirement_range.end.line
        || dependency.range.end.line == dependency.requirement_range.end.line
            && dependency.range.end.character >= dependency.requirement_range.end.character;

    starts_before_requirement && ends_after_requirement
}

fn is_cargo_sortable_group(group: &str) -> bool {
    CARGO_DEPENDENCY_GROUPS.contains(&group)
        || group == "workspace.dependencies"
        || (group.starts_with("target.")
            && CARGO_DEPENDENCY_GROUPS
                .iter()
                .any(|dependency_group| group.ends_with(dependency_group)))
}

fn is_deno_sortable_group(group: &str) -> bool {
    group == "imports" || group.starts_with("scopes.")
}

fn is_npm_sortable_group(group: &str) -> bool {
    NPM_DEPENDENCY_GROUPS.contains(&group)
        || matches!(
            group,
            "imports"
                | "jspm.dependencies"
                | "jspm.devDependencies"
                | "jspm.peerDependencies"
                | "jspm.optionalDependencies"
                | "workspaces.catalog"
        )
        || group.starts_with("scopes.")
        || group.starts_with("workspaces.catalogs.")
        || group.starts_with("catalogs.")
        || is_package_extension_group(group)
}

fn is_maven_sortable_group(group: &str) -> bool {
    matches!(
        group,
        "project.dependencies.dependency" | "project.dependencyManagement.dependencies.dependency"
    )
}

fn is_package_extension_group(group: &str) -> bool {
    (group.starts_with("packageExtensions.") || group.starts_with("pnpm.packageExtensions."))
        && NPM_DEPENDENCY_GROUPS
            .iter()
            .any(|dependency_group| group.ends_with(dependency_group))
}

fn sortable_dependencies(dependencies: &[Dependency]) -> Vec<&Dependency> {
    dependencies
        .iter()
        .filter(|dependency| is_sortable_dependency(dependency))
        .collect()
}

fn has_sortable_group<'a>(dependencies: impl Iterator<Item = &'a Dependency>) -> bool {
    let mut group_counts = BTreeMap::<&str, usize>::new();

    for dependency in dependencies {
        let count = group_counts.entry(dependency.group.as_str()).or_insert(0);
        *count += 1;
        if *count > 1 {
            return true;
        }
    }

    false
}

fn dependency_lines_are_unique<'a>(dependencies: impl Iterator<Item = &'a Dependency>) -> bool {
    let mut lines = BTreeSet::new();
    for dependency in dependencies {
        let line = dependency_start_line(dependency);
        if !lines.insert(line) {
            return false;
        }
    }
    true
}

fn has_sortable_span(dependency: &Dependency) -> bool {
    dependency.ecosystem == Ecosystem::Maven
        || dependency_start_line(dependency) == dependency_end_line(dependency)
}
