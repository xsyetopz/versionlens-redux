mod commit;
mod name;

pub(super) use commit::is_commit_sha;
use name::github_dependency_name;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(super) enum GitHubReferenceKind {
    Shortcut,
    Url,
}

pub(super) struct GitHubReference<'a> {
    pub(super) name: &'a str,
    pub(super) fragment: Option<&'a str>,
    pub(super) kind: GitHubReferenceKind,
}

pub(super) fn github_reference(value: &str) -> Option<GitHubReference<'_>> {
    let (base, fragment) = value
        .split_once('#')
        .map_or((value, None), |(base, fragment)| (base, Some(fragment)));
    let name = github_dependency_name(base)?;
    let kind = if is_github_shortcut(base) {
        GitHubReferenceKind::Shortcut
    } else {
        GitHubReferenceKind::Url
    };
    Some(GitHubReference {
        name,
        fragment,
        kind,
    })
}

fn is_github_shortcut(value: &str) -> bool {
    value
        .get(.."github:".len())
        .is_some_and(|head| head.eq_ignore_ascii_case("github:"))
        || (!value.contains(':') && !value.contains("://"))
}
