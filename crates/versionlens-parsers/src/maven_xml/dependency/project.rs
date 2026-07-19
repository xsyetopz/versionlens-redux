use versionlens_model::Dependency;

use super::super::nodes::{XmlNode, text_range};
use versionlens_model::Ecosystem::Maven;

pub(super) fn project_version_dependency(text: &str, node: &XmlNode) -> Dependency {
    Dependency {
        name: "version".to_owned(),
        requirement: node.text.as_str().to_owned(),
        ecosystem: Maven,
        group: node.path.as_str().to_owned(),
        hosted_url: None,
        hosted_name: None,
        range: text_range(text, node),
        requirement_range: text_range(text, node),
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
    }
}
