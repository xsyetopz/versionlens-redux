use super::encoding::encode_component;

pub(super) fn github_tags_url(name: &str) -> Option<String> {
    if name.starts_with('@') {
        return None;
    }
    let (owner, repo) = name.split_once('/')?;
    if owner.is_empty() || repo.is_empty() || repo.contains('/') {
        return None;
    }
    Some(format!(
        "https://api.github.com/repos/{}/{}/tags",
        encode_component(owner),
        encode_component(repo)
    ))
}
