use super::nodes::{collect_nodes, texts_from_nodes};

const METADATA_VERSION_PATH: &str = "metadata.versioning.versions.version";

pub fn parse_maven_metadata_versions(text: &str) -> Vec<String> {
    let Some(nodes) = collect_nodes(text) else {
        return Vec::new();
    };
    texts_from_nodes(&nodes, METADATA_VERSION_PATH)
}
