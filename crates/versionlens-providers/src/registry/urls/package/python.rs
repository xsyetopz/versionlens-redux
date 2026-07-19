use crate::python::normalized_project_name;
use crate::registry::endpoint::is_python_simple_base;

pub(in crate::registry::urls) fn python_registry_url(name: &str) -> String {
    format!("https://pypi.org/rss/project/{name}/releases.xml")
}

pub(in crate::registry::urls) fn python_registry_url_with_base(
    base_url: &str,
    name: &str,
) -> String {
    if !is_python_simple_base(base_url) {
        return base_url.to_owned();
    }

    format!(
        "{}/{}/",
        base_url.trim_end_matches('/'),
        normalized_project_name(name)
    )
}

pub fn python_package_json_url_template(base_url: &str) -> String {
    format!("{}/{{name}}/json", base_url.trim_end_matches('/'))
}
