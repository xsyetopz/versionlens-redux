pub(super) fn is_fixed_spec(requirement: &str) -> bool {
    let requirement = requirement.trim();
    requirement.starts_with("catalog:")
        || requirement.starts_with("workspace:")
        || is_direct_url_requirement(requirement)
}

fn is_direct_url_requirement(requirement: &str) -> bool {
    requirement.contains("://")
}
