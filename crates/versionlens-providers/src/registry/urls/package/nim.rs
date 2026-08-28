use super::super::encoding::encode_component;
use super::super::trim_end_slash;
use super::github::github_registry_url;

pub(in crate::registry::urls) fn nim_registry_url(name: &str) -> String {
    if name.contains('/') {
        return github_registry_url(name);
    }
    nim_registry_url_with_base(
        "https://raw.githubusercontent.com/nim-lang/packages/master/packages.json",
        name,
    )
}

pub(in crate::registry::urls) fn nim_registry_url_with_base(base_url: &str, name: &str) -> String {
    let base_url = trim_end_slash(base_url);
    if crate::path(base_url)
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
    {
        return base_url.to_owned();
    }
    format!("{base_url}/{}", encode_component(name))
}
