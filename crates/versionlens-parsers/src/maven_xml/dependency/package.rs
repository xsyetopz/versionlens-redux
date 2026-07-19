use crate::positions::offset_range;
use versionlens_model::Dependency;

use super::super::nodes::{XmlNode, child_named, direct_children, text_range};
use super::{MavenParseContext, resolve_property};
use versionlens_model::Ecosystem::Maven;

pub(super) fn package_dependency(
    context: &MavenParseContext<'_>,
    node: &XmlNode,
) -> Option<Dependency> {
    let children = direct_children(node, context.nodes);
    let group = resolve_property(
        child_named(&children, "groupId")?,
        context.nodes,
        context.properties,
    );
    let artifact = resolve_property(
        child_named(&children, "artifactId")?,
        context.nodes,
        context.properties,
    );
    let version = resolve_property(
        child_named(&children, "version")?,
        context.nodes,
        context.properties,
    );
    if group.text.is_empty() || artifact.text.is_empty() || version.text.is_empty() {
        return None;
    }

    Some(Dependency {
        name: format!("{}:{}", group.text, artifact.text),
        requirement: version.text.as_str().to_owned(),
        ecosystem: Maven,
        group: node.path.as_str().to_owned(),
        hosted_url: None,
        hosted_name: None,
        range: offset_range(context.text, node.open_start, node.close_end),
        requirement_range: text_range(context.text, version),
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
    })
}
