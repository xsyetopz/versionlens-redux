use super::super::DotnetTagKind::{Empty as DotnetTagEmpty, Start as DotnetTagStart};
use quick_xml::events::BytesStart;
use quick_xml::events::BytesText;
use std::slice::from_ref;

use crate::path_patterns::path_or_member_enabled;
use crate::positions::offset_range;
use versionlens_model::Dependency;

use super::super::attributes::{attr_value, tag_bounds, version_insert};
use super::super::dependency::{dependencies_from_tag, is_package_tag, project_version_dependency};
use super::super::{
    DotnetEventContext, DotnetTagKind, OpenProjectVersion, event_name, event_name_from_bytes,
};
use versionlens_model::Ecosystem::Dotnet;

type DotnetPackageDependency = Option<Dependency>;
type DotnetXmlEvent<'a> = BytesStart<'a>;

pub(super) struct DotnetXmlCollector<'a> {
    stack: Vec<String>,
    open_project_version: Option<OpenProjectVersion>,
    open_package_version: Option<OpenPackageVersion>,
    dependencies: Vec<Dependency>,
    dependency_paths: Vec<&'a str>,
}

struct OpenPackageVersion {
    end_name: String,
    fallback: Dependency,
    child: Option<OpenChildPackageVersion>,
    child_dependency_pushed: bool,
}

struct OpenChildPackageVersion {
    text_start: usize,
    value: String,
}

impl<'a> DotnetXmlCollector<'a> {
    pub(super) fn start_tag(
        &mut self,
        context: &DotnetEventContext<'_>,
        event: &DotnetXmlEvent<'_>,
    ) {
        let Some(name) = event_name(event) else {
            return;
        };
        self.collect_tag_dependencies(context, event, DotnetTagStart);
        if is_project_version_path(&self.stack, &name) {
            self.open_project_version = Some(OpenProjectVersion {
                text_start: context.span.end,
                value: "".to_owned(),
            });
        }
        if is_package_child_version_path(&self.stack, &name) {
            if let Some(open) = &mut self.open_package_version {
                open.child = Some(OpenChildPackageVersion {
                    text_start: context.span.end,
                    value: "".to_owned(),
                });
            }
        }
        if is_package_tag(&name) && self.open_package_version.is_none() {
            self.open_package_version = pending_package_version(context, &name);
        }
        if is_packages_config_package_path(&self.stack, &name)
            && let Some(dependency) = packages_config_dependency(context)
        {
            self.push_enabled_dependency(dependency);
        }
        self.stack.push(name);
    }

    pub(super) fn empty_tag(
        &mut self,
        context: &DotnetEventContext<'_>,
        event: &DotnetXmlEvent<'_>,
    ) {
        if let Some(name) = event_name(event)
            && is_packages_config_package_path(&self.stack, &name)
            && let Some(dependency) = packages_config_dependency(context)
        {
            self.push_enabled_dependency(dependency);
        }
        self.collect_tag_dependencies(context, event, DotnetTagEmpty);
    }

    pub(super) fn text(&mut self, event: &BytesText<'_>) {
        let Ok(value) = event.decode() else {
            return;
        };
        if let Some(open) = &mut self.open_project_version {
            open.value.push_str(&value);
        }
        if let Some(child) = self
            .open_package_version
            .as_mut()
            .and_then(|open| open.child.as_mut())
        {
            child.value.push_str(&value);
        }
    }

    pub(super) fn end_tag(&mut self, text: &str, end_name: &[u8]) {
        if let Some(name) = event_name_from_bytes(end_name) {
            let is_project_version = matches!(name.as_str(), "Version" | "AssemblyVersion");
            if let Some(open) = self.open_project_version.take()
                && is_project_version
            {
                self.push_enabled_dependency(project_version_dependency(text, name, &open));
                self.stack.pop();
                return;
            }
            if name == "Version" {
                if let Some(dependency) = self.package_child_version_dependency(text) {
                    self.push_enabled_dependency(dependency);
                }
                self.stack.pop();
                return;
            }
            if let Some(open) = self.open_package_version.take() {
                if name == open.end_name {
                    if !open.child_dependency_pushed {
                        self.push_enabled_dependency(open.fallback);
                    }
                    self.stack.pop();
                    return;
                }
                self.open_package_version = Some(open);
            }
        }
        self.stack.pop();
    }

    fn package_child_version_dependency(&mut self, text: &str) -> DotnetPackageDependency {
        let open = self.open_package_version.as_mut()?;
        let child = open.child.take()?;
        let value = child.value.trim();
        let value_start = text[child.text_start..]
            .find(value)
            .map_or(child.text_start, |offset| child.text_start + offset);
        open.child_dependency_pushed = true;

        Some(Dependency {
            name: open.fallback.name.as_str().to_owned(),
            requirement: value.to_owned(),
            ecosystem: Dotnet,
            group: open.fallback.group.as_str().to_owned(),
            hosted_url: None,
            hosted_name: None,
            range: open.fallback.range,
            requirement_range: offset_range(text, value_start, value_start + value.len()),
            requirement_prefix: "".to_owned(),
            requirement_suffix: "".to_owned(),
        })
    }

    fn collect_tag_dependencies(
        &mut self,
        context: &DotnetEventContext<'_>,
        event: &DotnetXmlEvent<'_>,
        tag_kind: DotnetTagKind,
    ) {
        for dependency in dependencies_from_tag(context, event, tag_kind) {
            self.push_enabled_dependency(dependency);
        }
    }

    fn push_enabled_dependency(&mut self, dependency: Dependency) -> bool {
        if dependency_path_enabled(&self.dependency_paths, &dependency) {
            self.dependencies.push(dependency);
            return true;
        }
        false
    }

    pub(super) fn finish(mut self) -> Vec<Dependency> {
        sort_dependencies_by_path_order(&mut self.dependencies, &self.dependency_paths);
        self.dependencies
    }
}

fn dependency_path_enabled(dependency_paths: &[&str], dependency: &Dependency) -> bool {
    let group = dependency_property_group(dependency);
    path_or_member_enabled(dependency_paths, &group, Some(&dependency.name))
}

fn dependency_property_group(dependency: &Dependency) -> String {
    match dependency.group.as_str() {
        "PropertyGroup" => format!("Project.PropertyGroup.{}", dependency.name),
        "packages.package" => "packages.package".to_owned(),
        "Sdk" | "Project.Sdk" => "Project.Sdk".to_owned(),
        group => format!("Project.ItemGroup.{group}"),
    }
}

fn sort_dependencies_by_path_order(dependencies: &mut [Dependency], dependency_paths: &[&str]) {
    dependencies.sort_by_key(|dependency| dependency_path_rank(dependency_paths, dependency));
}

fn dependency_path_rank(dependency_paths: &[&str], dependency: &Dependency) -> usize {
    let group = dependency_property_group(dependency);
    dependency_paths
        .iter()
        .position(|path| path_or_member_enabled(from_ref(path), &group, Some(&dependency.name)))
        .unwrap_or(dependency_paths.len())
}

fn is_project_version_path(stack: &[String], name: &str) -> bool {
    matches!(name, "Version" | "AssemblyVersion")
        && stack
            .iter()
            .map(|value| value.as_str())
            .eq(["Project", "PropertyGroup"])
}

fn is_package_child_version_path(stack: &[String], name: &str) -> bool {
    name == "Version"
        && stack.len() == 3
        && stack[0] == "Project"
        && stack[1] == "ItemGroup"
        && is_package_tag(&stack[2])
}

fn is_packages_config_package_path(stack: &[String], name: &str) -> bool {
    name == "package" && stack.iter().map(|value| value.as_str()).eq(["packages"])
}

fn packages_config_dependency(context: &DotnetEventContext<'_>) -> DotnetPackageDependency {
    let (tag_start, tag_end) = tag_bounds(context.text, context.span.start, context.span.end);
    let tag = context.text.get(tag_start..tag_end)?;
    let name = attr_value(tag, "id")?;
    let version = attr_value(tag, "version")?;
    let name_start = tag_start + name.range.start;
    let version_start = tag_start + version.range.start;

    Some(Dependency {
        name: name.value,
        requirement: version.value,
        ecosystem: Dotnet,
        group: "packages.package".to_owned(),
        hosted_url: None,
        hosted_name: None,
        range: offset_range(context.text, name_start, name_start + name.len),
        requirement_range: offset_range(context.text, version_start, version_start + version.len),
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
    })
}

fn pending_package_version(
    context: &DotnetEventContext<'_>,
    group: &str,
) -> Option<OpenPackageVersion> {
    missing_package_version_dependency(context, group).map(|fallback| OpenPackageVersion {
        end_name: group.to_owned(),
        fallback,
        child: None,
        child_dependency_pushed: false,
    })
}

fn missing_package_version_dependency(
    context: &DotnetEventContext<'_>,
    group: &str,
) -> DotnetPackageDependency {
    ["Include", "Update"].iter().find_map(|name_attr| {
        missing_package_version_dependency_with_name(context, group, name_attr)
    })
}

fn missing_package_version_dependency_with_name(
    context: &DotnetEventContext<'_>,
    group: &str,
    name_attr: &str,
) -> DotnetPackageDependency {
    let (tag_start, tag_end) = tag_bounds(context.text, context.span.start, context.span.end);
    let tag = context.text.get(tag_start..tag_end)?;
    if attr_value(tag, "Version").is_some() || attr_value(tag, "VersionOverride").is_some() {
        return None;
    }
    let name = attr_value(tag, name_attr)?;
    let name_start = tag_start + name.range.start;
    let (insert_offset, separator) = version_insert(tag)?;

    Some(Dependency {
        name: name.value,
        requirement: "*".to_owned(),
        ecosystem: Dotnet,
        group: group.to_owned(),
        hosted_url: None,
        hosted_name: None,
        range: offset_range(context.text, name_start, name_start + name.len),
        requirement_range: offset_range(
            context.text,
            tag_start + insert_offset,
            tag_start + insert_offset,
        ),
        requirement_prefix: format!("{separator}Version=\""),
        requirement_suffix: "\"".to_owned(),
    })
}

pub(super) fn dotnet_xml_collector<'a>(dependency_paths: Vec<&'a str>) -> DotnetXmlCollector<'a> {
    DotnetXmlCollector {
        stack: vec![],
        open_project_version: None,
        open_package_version: None,
        dependencies: vec![],
        dependency_paths,
    }
}
