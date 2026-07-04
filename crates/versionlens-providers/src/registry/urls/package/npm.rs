use super::super::hosted::github_tags_url;
use super::super::trim_end_slash;

pub(in crate::registry::urls) fn npm_registry_url(name: &str) -> String {
    github_tags_url(name)
        .unwrap_or_else(|| format!("https://registry.npmjs.org/{}", npm_package_path(name)))
}

pub(in crate::registry::urls) fn npm_registry_url_with_base(base_url: &str, name: &str) -> String {
    format!("{}/{}", trim_end_slash(base_url), npm_package_path(name))
}

fn npm_package_path(name: &str) -> String {
    name.replace('/', "%2f")
}
