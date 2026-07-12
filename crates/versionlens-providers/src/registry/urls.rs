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
use encoding::encode_unsafe_url_bytes;
use go::{go_registry_url, go_registry_url_with_base};
use maven::{maven_registry_url, maven_registry_url_with_base};
use package::{
    ansible_registry_url, ansible_registry_url_with_base, bazel_registry_url,
    bazel_registry_url_with_base, cargo_registry_url, cargo_registry_url_with_base,
    cocoapods_registry_url, cocoapods_registry_url_with_base, composer_registry_url,
    composer_registry_url_with_base, conan_registry_url, conan_registry_url_with_base,
    cpan_registry_url, cpan_registry_url_with_base, cpp_registry_url, cpp_registry_url_with_base,
    cran_registry_url, cran_registry_url_with_base, deno_registry_url, deno_registry_url_with_base,
    dub_registry_url, dub_registry_url_with_base, hackage_registry_url,
    hackage_registry_url_with_base, haxelib_registry_url, haxelib_registry_url_with_base,
    helm_registry_url, helm_registry_url_with_base, hex_registry_url, hex_registry_url_with_base,
    julia_registry_url, julia_registry_url_with_base, luarocks_registry_url,
    luarocks_registry_url_with_base, nim_registry_url, nim_registry_url_with_base,
    nix_registry_url, nix_registry_url_with_base, npm_registry_url, npm_registry_url_with_base,
    opam_registry_url, opam_registry_url_with_base, pub_registry_url, pub_registry_url_with_base,
    python_registry_url, python_registry_url_with_base, ruby_registry_url,
    ruby_registry_url_with_base, swift_registry_url, swift_registry_url_with_base,
    terraform_registry_url, terraform_registry_url_with_base, unity_registry_url,
    unity_registry_url_with_base, vcpkg_registry_url, vcpkg_registry_url_with_base,
    zig_registry_url, zig_registry_url_with_base,
};
use template::template_registry_url;

pub use docker::{
    docker_hub_body_has_next_page, docker_hub_tags_page_url, merge_docker_hub_response_pages,
};
pub use dotnet::dotnet_package_url_from_service_index;
pub use package::{ansible_role_registry_url_with_base, python_package_json_url_template};

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
    hex_registry_url,
    opam_registry_url,
    hackage_registry_url,
    julia_registry_url,
    cran_registry_url,
    conan_registry_url,
    vcpkg_registry_url,
    swift_registry_url,
    zig_registry_url,
    nim_registry_url,
    luarocks_registry_url,
    cpan_registry_url,
    haxelib_registry_url,
    terraform_registry_url,
    helm_registry_url,
    ansible_registry_url,
    bazel_registry_url,
    nix_registry_url,
    unity_registry_url,
    cocoapods_registry_url,
    cpp_registry_url,
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
    hex_registry_url_with_base,
    opam_registry_url_with_base,
    hackage_registry_url_with_base,
    julia_registry_url_with_base,
    cran_registry_url_with_base,
    conan_registry_url_with_base,
    vcpkg_registry_url_with_base,
    swift_registry_url_with_base,
    zig_registry_url_with_base,
    nim_registry_url_with_base,
    luarocks_registry_url_with_base,
    cpan_registry_url_with_base,
    haxelib_registry_url_with_base,
    terraform_registry_url_with_base,
    helm_registry_url_with_base,
    ansible_registry_url_with_base,
    bazel_registry_url_with_base,
    nix_registry_url_with_base,
    unity_registry_url_with_base,
    cocoapods_registry_url_with_base,
    cpp_registry_url_with_base,
];

pub fn registry_url(ecosystem: Ecosystem, name: &str) -> String {
    let name = name.trim();
    encode_unsafe_url_bytes(&DEFAULT_REGISTRY_URLS[ecosystem as usize](name))
}

pub fn registry_url_with_base(ecosystem: Ecosystem, name: &str, base_url: Option<&str>) -> String {
    let name = name.trim();
    let Some(base_url) = base_url.map(str::trim).filter(|url| !url.is_empty()) else {
        return registry_url(ecosystem, name);
    };

    if let Some(url) = template_registry_url(ecosystem, name, base_url) {
        return encode_unsafe_url_bytes(&url);
    }

    encode_unsafe_url_bytes(&registry_url_from_base(ecosystem, name, base_url))
}

fn registry_url_from_base(ecosystem: Ecosystem, name: &str, base_url: &str) -> String {
    CUSTOM_REGISTRY_URLS[ecosystem as usize](base_url, name)
}
