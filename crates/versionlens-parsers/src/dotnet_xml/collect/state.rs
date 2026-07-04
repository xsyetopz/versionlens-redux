use quick_xml::events::BytesStart;

use crate::model::Dependency;
use crate::path_patterns::path_or_member_enabled;

use super::super::dependency::{dependencies_from_tag, project_version_dependency};
use super::super::{
    DotnetEventContext, DotnetTagKind, OpenProjectVersion, event_name, event_name_from_bytes,
};

pub(super) struct DotnetXmlCollector<'a> {
    stack: Vec<String>,
    open_project_version: Option<OpenProjectVersion>,
    dependencies: Vec<Dependency>,
    dependency_paths: Vec<&'a str>,
}

impl<'a> DotnetXmlCollector<'a> {
    pub(super) fn new(dependency_paths: Vec<&'a str>) -> Self {
        Self {
            stack: Vec::new(),
            open_project_version: None,
            dependencies: Vec::new(),
            dependency_paths,
        }
    }

    pub(super) fn start_tag(&mut self, context: &DotnetEventContext<'_>, event: &BytesStart<'_>) {
        let Some(name) = event_name(event) else {
            return;
        };
        self.collect_tag_dependencies(context, event, DotnetTagKind::Start);
        if is_project_version_path(&self.stack, &name) {
            self.open_project_version = Some(OpenProjectVersion {
                text_start: context.span.end,
                value: String::new(),
            });
        }
        self.stack.push(name);
    }

    pub(super) fn empty_tag(&mut self, context: &DotnetEventContext<'_>, event: &BytesStart<'_>) {
        self.collect_tag_dependencies(context, event, DotnetTagKind::Empty);
    }

    pub(super) fn text(&mut self, event: &quick_xml::events::BytesText<'_>) {
        let Ok(value) = event.decode() else {
            return;
        };
        if let Some(open) = &mut self.open_project_version {
            open.value.push_str(&value);
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
        }
        self.stack.pop();
    }

    fn collect_tag_dependencies(
        &mut self,
        context: &DotnetEventContext<'_>,
        event: &BytesStart<'_>,
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
        .position(|path| {
            path_or_member_enabled(std::slice::from_ref(path), &group, Some(&dependency.name))
        })
        .unwrap_or(dependency_paths.len())
}

fn is_project_version_path(stack: &[String], name: &str) -> bool {
    matches!(name, "Version" | "AssemblyVersion")
        && stack
            .iter()
            .map(String::as_str)
            .eq(["Project", "PropertyGroup"])
}
