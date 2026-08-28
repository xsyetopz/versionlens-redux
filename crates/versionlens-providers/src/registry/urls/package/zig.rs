use super::super::encoding::encode_component;
use super::super::trim_end_slash;
use super::github::github_registry_url;

pub(in crate::registry::urls) fn zig_registry_url(name: &str) -> String {
    if name.contains('/') {
        return github_registry_url(name);
    }
    zig_registry_url_with_base("https://pkg.ziglang.org", name)
}

pub(in crate::registry::urls) fn zig_registry_url_with_base(base_url: &str, name: &str) -> String {
    let base_url = trim_end_slash(base_url);
    format!("{base_url}/{}", encode_component(name))
}
