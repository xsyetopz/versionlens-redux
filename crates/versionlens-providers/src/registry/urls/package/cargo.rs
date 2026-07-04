use super::super::trim_end_slash;

pub(in crate::registry::urls) fn cargo_registry_url(name: &str) -> String {
    cargo_registry_url_with_base("https://crates.io/api/v1/crates", name)
}

pub(in crate::registry::urls) fn cargo_registry_url_with_base(
    base_url: &str,
    name: &str,
) -> String {
    format!("{}/{name}/versions", trim_end_slash(base_url))
}
