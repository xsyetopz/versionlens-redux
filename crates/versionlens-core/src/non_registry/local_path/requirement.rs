pub(super) fn local_requirement_path(requirement: &str) -> Option<&str> {
    let requirement = requirement.trim();
    let path = requirement
        .strip_prefix("file:")
        .or_else(|| requirement.strip_prefix("link:"))
        .or_else(|| requirement.strip_prefix("path:"))
        .unwrap_or(requirement);

    (is_local_requirement(requirement) && !path.is_empty()).then_some(path)
}

fn is_local_requirement(requirement: &str) -> bool {
    requirement.starts_with("file:")
        || requirement.starts_with("link:")
        || requirement.starts_with("path:")
        || requirement.starts_with('/')
        || requirement.starts_with("./")
        || requirement.starts_with("../")
        || requirement.starts_with("~/")
}
