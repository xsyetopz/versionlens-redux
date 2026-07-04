use versionlens_parsers::Ecosystem;

mod docker;
mod dotnet;
mod encoding;
mod go;
mod hosted;
mod maven;
mod package;
mod template;

use docker::{docker_registry_url, docker_registry_url_with_base};
use dotnet::{dotnet_default_registry_url, dotnet_package_url};
use go::{go_registry_url, go_registry_url_with_base};
use maven::{maven_registry_url, maven_registry_url_with_base};
use package::{
    cargo_registry_url, cargo_registry_url_with_base, composer_registry_url,
    composer_registry_url_with_base, deno_registry_url, deno_registry_url_with_base,
    dub_registry_url, dub_registry_url_with_base, npm_registry_url, npm_registry_url_with_base,
    pub_registry_url, pub_registry_url_with_base, python_registry_url,
    python_registry_url_with_base, ruby_registry_url, ruby_registry_url_with_base,
};
use template::template_registry_url;

pub use docker::{
    docker_hub_body_has_next_page, docker_hub_tags_page_url, merge_docker_hub_response_pages,
};
pub use dotnet::dotnet_package_url_from_service_index;
pub use package::python_package_json_url_template;

pub(super) fn trim_end_slash(value: &str) -> &str {
    value.trim_end_matches('/')
}

type RegistryUrlBuilder = fn(&str) -> String;
type RegistryUrlWithBaseBuilder = fn(&str, &str) -> String;

const DEFAULT_REGISTRY_URLS: &[RegistryUrlBuilder] = &[
    cargo_registry_url,
    composer_registry_url,
    deno_registry_url,
    dotnet_default_registry_url,
    docker_registry_url,
    dub_registry_url,
    go_registry_url,
    maven_registry_url,
    npm_registry_url,
    python_registry_url,
    pub_registry_url,
    ruby_registry_url,
];

const CUSTOM_REGISTRY_URLS: &[RegistryUrlWithBaseBuilder] = &[
    cargo_registry_url_with_base,
    composer_registry_url_with_base,
    deno_registry_url_with_base,
    dotnet_package_url,
    docker_registry_url_with_base,
    dub_registry_url_with_base,
    go_registry_url_with_base,
    maven_registry_url_with_base,
    npm_registry_url_with_base,
    python_registry_url_with_base,
    pub_registry_url_with_base,
    ruby_registry_url_with_base,
];

pub fn registry_url(ecosystem: Ecosystem, name: &str) -> String {
    let name = name.trim();
    DEFAULT_REGISTRY_URLS[ecosystem as usize](name)
}

pub fn registry_url_with_base(ecosystem: Ecosystem, name: &str, base_url: Option<&str>) -> String {
    let name = name.trim();
    let Some(base_url) = base_url.map(str::trim).filter(|url| !url.is_empty()) else {
        return registry_url(ecosystem, name);
    };

    if let Some(url) = template_registry_url(ecosystem, name, base_url) {
        return url;
    }

    registry_url_from_base(ecosystem, name, base_url)
}

fn registry_url_from_base(ecosystem: Ecosystem, name: &str, base_url: &str) -> String {
    CUSTOM_REGISTRY_URLS[ecosystem as usize](base_url, name)
}
