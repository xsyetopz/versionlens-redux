use crate::model::{Dependency, Ecosystem};

use super::super::nodes::{XmlNode, text_range};

pub(super) fn project_version_dependency(text: &str, node: &XmlNode) -> Dependency {
    Dependency {
        name: "version".to_owned(),
        requirement: node.text.as_str().to_owned(),
        ecosystem: Ecosystem::Maven,
        group: node.path.as_str().to_owned(),
        hosted_url: None,
        hosted_name: None,
        range: text_range(text, node),
        requirement_range: text_range(text, node),
        requirement_prefix: String::new(),
        requirement_suffix: String::new(),
    }
}
