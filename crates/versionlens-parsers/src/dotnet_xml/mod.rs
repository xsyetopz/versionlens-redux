use quick_xml::events::BytesStart;
use std::str;
use versionlens_model::Dependency;

mod attributes;
mod collect;
mod dependency;

use collect::collect_dotnet_xml_dependencies;

pub(crate) fn parse_dotnet_xml_with_paths(
    text: &str,
    dependency_paths: &[&str],
) -> Vec<Dependency> {
    collect_dotnet_xml_dependencies(text, dependency_paths.to_vec())
}

pub(super) struct DotnetEventContext<'a> {
    pub(super) text: &'a str,
    pub(super) span: DotnetTagSpan,
}

pub(super) struct DotnetTagSpan {
    pub(super) start: usize,
    pub(super) end: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DotnetTagKind {
    Empty,
    Start,
}

pub(super) struct OpenProjectVersion {
    pub(super) text_start: usize,
    pub(super) value: String,
}

pub(super) fn event_name(event: &BytesStart<'_>) -> Option<String> {
    event_name_from_bytes(event.name().as_ref().as_bytes())
}

pub(super) fn event_name_from_bytes(bytes: &[u8]) -> Option<String> {
    str::from_utf8(bytes).ok().map(|value| value.to_owned())
}

#[cfg(test)]
mod tests;
