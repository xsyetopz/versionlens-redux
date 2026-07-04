use super::super::trim_end_slash;

pub(in crate::registry::urls) fn pub_registry_url(name: &str) -> String {
    format!("https://pub.dev/api/packages/{name}")
}

pub(in crate::registry::urls) fn pub_registry_url_with_base(base_url: &str, name: &str) -> String {
    format!("{}/{name}", trim_end_slash(base_url))
}
