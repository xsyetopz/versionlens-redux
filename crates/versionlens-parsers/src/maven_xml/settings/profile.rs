use super::super::nodes::{XmlNode, child_named, direct_children};

pub(super) fn active_profile_ids(nodes: &[XmlNode]) -> Vec<String> {
    nodes
        .iter()
        .filter(|node| node.path == "settings.activeProfiles.activeProfile")
        .filter(|node| !node.text.is_empty())
        .map(|node| node.text.as_str().to_owned())
        .collect()
}

pub(super) fn profile_is_active(
    node: &XmlNode,
    nodes: &[XmlNode],
    active_profiles: &[String],
) -> bool {
    active_profiles.is_empty()
        || profile_id_for_node(node, nodes)
            .is_some_and(|profile_id| active_profiles.iter().any(|active| active == profile_id))
}

fn profile_id_for_node<'a>(node: &XmlNode, nodes: &'a [XmlNode]) -> Option<&'a str> {
    let profile = nodes.iter().find(|candidate| {
        candidate.path == "settings.profiles.profile"
            && candidate.open_start < node.open_start
            && candidate.close_end > node.close_end
    })?;
    let children = direct_children(profile, nodes);
    child_named(&children, "id").map(|id| id.text.as_str())
}
