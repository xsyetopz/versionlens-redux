use super::github::github_registry_url;

pub(in crate::registry::urls) fn nix_registry_url(name: &str) -> String {
    let identity = if name.contains('/') {
        name.to_owned()
    } else {
        format!("NixOS/{name}")
    };
    github_registry_url(&identity)
}

pub(in crate::registry::urls) fn nix_registry_url_with_base(base_url: &str, name: &str) -> String {
    if base_url.starts_with("https://api.github.com/repos") {
        return nix_registry_url(name);
    }
    nix_registry_url(name)
}
