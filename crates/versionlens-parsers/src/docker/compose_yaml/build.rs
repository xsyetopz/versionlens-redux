use marked_yaml::types::{MarkedScalarNode, Node};

use crate::{
    model::{Dependency, Ecosystem},
    positions::offset_range,
    yaml::scalar_range,
};

pub(super) fn build_dependency(text: &str, value: &Node) -> Option<Dependency> {
    match value {
        Node::Scalar(context) => build_path_dependency(text, context, "dockerfile"),
        Node::Mapping(map) => {
            let context = map.get_scalar("context")?;
            let dockerfile = map
                .get_scalar("dockerfile")
                .map(MarkedScalarNode::as_str)
                .unwrap_or("dockerfile");
            build_path_dependency(text, context, dockerfile)
        }
        Node::Sequence(_) => None,
    }
}

fn build_path_dependency(
    text: &str,
    context: &MarkedScalarNode,
    dockerfile: &str,
) -> Option<Dependency> {
    let name = compose_build_path(context.as_str(), dockerfile);
    let requirement = compose_build_path(context.as_str(), dockerfile);
    let value_range = scalar_range(text, context)?;
    Some(Dependency {
        name,
        requirement,
        ecosystem: Ecosystem::Docker,
        group: "services.build".to_owned(),
        hosted_url: None,
        hosted_name: None,
        range: offset_range(text, value_range.start, value_range.end),
        requirement_range: offset_range(text, value_range.start, value_range.end),
        requirement_prefix: String::new(),
        requirement_suffix: String::new(),
    })
}

fn compose_build_path(context: &str, dockerfile: &str) -> String {
    format!("{context}/{dockerfile}")
}
