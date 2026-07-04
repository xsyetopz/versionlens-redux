use super::super::trim_end_slash;

pub(in crate::registry::urls) fn composer_registry_url(name: &str) -> String {
    format!("https://repo.packagist.org/p2/{name}.json")
}

pub(in crate::registry::urls) fn composer_registry_url_with_base(
    base_url: &str,
    name: &str,
) -> String {
    format!("{}/{name}.json", trim_end_slash(base_url))
}
