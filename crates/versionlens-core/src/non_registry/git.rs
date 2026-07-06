pub(super) fn is_git_dependency_requirement(requirement: &str) -> bool {
    let requirement = requirement.trim();
    requirement.starts_with("git+")
        || requirement.starts_with("git@")
        || requirement.starts_with("git://")
        || requirement.starts_with("ssh://")
        || crate::path(requirement)
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("git"))
        || is_hosted_git_url(requirement)
}

pub(super) fn is_unsupported_npm_git_requirement(requirement: &str) -> bool {
    is_git_dependency_requirement(requirement) && !is_hosted_git_requirement(requirement)
}

fn is_hosted_git_requirement(requirement: &str) -> bool {
    let requirement = requirement.trim();
    if is_hosted_git_url(requirement) {
        return true;
    }
    let requirement = requirement.strip_prefix("git+").unwrap_or(requirement);
    is_hosted_git_url(requirement) || is_hosted_scp_git_url(requirement)
}

fn is_hosted_git_url(requirement: &str) -> bool {
    url_host(requirement).is_some_and(is_hosted_git_host)
}

fn is_hosted_scp_git_url(requirement: &str) -> bool {
    requirement
        .strip_prefix("git@")
        .and_then(|rest| rest.split([':', '/']).next())
        .is_some_and(is_hosted_git_host)
}

fn url_host(requirement: &str) -> Option<&str> {
    ["https://", "http://", "ssh://", "git://"]
        .into_iter()
        .find_map(|scheme| requirement.strip_prefix(scheme))
        .and_then(|rest| rest.split('/').next())
        .and_then(|authority| authority.rsplit('@').next())
        .and_then(|host| host.split(':').next())
}

fn is_hosted_git_host(host: &str) -> bool {
    matches!(host, "github.com" | "gitlab.com" | "bitbucket.org")
}
