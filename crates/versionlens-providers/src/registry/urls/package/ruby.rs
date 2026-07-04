use super::super::hosted::github_tags_url;

pub(in crate::registry::urls) fn ruby_registry_url(name: &str) -> String {
    github_tags_url(name)
        .unwrap_or_else(|| format!("https://rubygems.org/api/v1/versions/{name}.json"))
}

pub(in crate::registry::urls) fn ruby_registry_url_with_base(
    base_url: &str,
    _name: &str,
) -> String {
    base_url.to_owned()
}
