use crate::positions::offset_range;
use crate::yaml::scalar_range;
use marked_yaml::types::MarkedScalarNode;
use marked_yaml::types::Node;
use marked_yaml::types::Node::{
    Mapping as YamlMapping, Scalar as YamlScalar, Sequence as YamlSequence,
};

use versionlens_model::Dependency;
use versionlens_model::Ecosystem::Docker;

pub(super) fn build_dependency(text: &str, value: &Node) -> Option<Dependency> {
    match value {
        YamlScalar(context) => build_path_dependency(text, context, "dockerfile"),
        YamlMapping(map) => {
            let context = map.get_scalar("context")?;
            let dockerfile = map
                .get_scalar("dockerfile")
                .map(|value| value.as_str())
                .unwrap_or("dockerfile");
            build_path_dependency(text, context, dockerfile)
        }
        YamlSequence(_) => None,
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
        ecosystem: Docker,
        group: "services.build".to_owned(),
        hosted_url: None,
        hosted_name: None,
        range: offset_range(text, value_range.start, value_range.end),
        requirement_range: offset_range(text, value_range.start, value_range.end),
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
    })
}

fn compose_build_path(context: &str, dockerfile: &str) -> String {
    format!("{context}/{dockerfile}")
}
