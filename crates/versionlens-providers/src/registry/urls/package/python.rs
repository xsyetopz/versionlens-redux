pub(in crate::registry::urls) fn python_registry_url(name: &str) -> String {
    format!("https://pypi.org/rss/project/{name}/releases.xml")
}

pub(in crate::registry::urls) fn python_registry_url_with_base(
    base_url: &str,
    _name: &str,
) -> String {
    base_url.to_owned()
}

pub fn python_package_json_url_template(base_url: &str) -> String {
    format!("{}/{{name}}/json", base_url.trim_end_matches('/'))
}
