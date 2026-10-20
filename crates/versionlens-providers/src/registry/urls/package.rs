use super::encoding::encode_component;
use super::hosted::github_tags_url;
use super::trim_end_slash;
use crate::python::normalized_project_name;
use crate::registry::endpoint::is_python_simple_base;
use versionlens_model::GithubRepository;

pub(super) fn ansible_registry_url(name: &str) -> String {
    ansible_registry_url_with_base("https://galaxy.ansible.com", name)
}

pub(super) fn ansible_registry_url_with_base(base_url: &str, name: &str) -> String {
    let (namespace, package) = name.split_once('.').unwrap_or(("", name));
    format!(
        "{}/api/v3/plugin/ansible/content/published/collections/index/{}/{}/versions/",
        trim_end_slash(base_url),
        encode_component(namespace),
        encode_component(package)
    )
}

pub fn ansible_role_registry_url_with_base(base_url: &str, name: &str) -> String {
    let (namespace, role) = name.split_once('.').unwrap_or(("", name));
    format!(
        "{}/api/v1/roles/?owner__username={}&name={}",
        trim_end_slash(base_url),
        encode_component(namespace),
        encode_component(role)
    )
}

pub(super) fn bazel_registry_url(name: &str) -> String {
    format!(
        "https://raw.githubusercontent.com/bazelbuild/bazel-central-registry/main/modules/{}/metadata.json",
        encode_component(name)
    )
}

pub(super) fn bazel_registry_url_with_base(base_url: &str, name: &str) -> String {
    format!(
        "{}/modules/{}/metadata.json",
        trim_end_slash(base_url),
        encode_component(name)
    )
}

pub(super) fn cargo_registry_url(name: &str) -> String {
    cargo_registry_url_with_base("https://crates.io/api/v1/crates", name)
}

pub(super) fn cargo_registry_url_with_base(base_url: &str, name: &str) -> String {
    format!("{}/{name}/versions", trim_end_slash(base_url))
}

pub(super) fn cocoapods_registry_url(name: &str) -> String {
    cocoapods_registry_url_with_base("https://trunk.cocoapods.org/api/v1/pods", name)
}

pub(super) fn cocoapods_registry_url_with_base(base_url: &str, name: &str) -> String {
    let root_name = name.split('/').next().unwrap_or(name);
    format!(
        "{}/{}",
        trim_end_slash(base_url),
        encode_component(root_name)
    )
}

pub(super) fn composer_registry_url(name: &str) -> String {
    format!("https://repo.packagist.org/p2/{name}.json")
}

pub(super) fn composer_registry_url_with_base(base_url: &str, name: &str) -> String {
    format!("{}/{name}.json", trim_end_slash(base_url))
}

pub(super) fn conan_registry_url(name: &str) -> String {
    conan_registry_url_with_base("https://center2.conan.io", name)
}

pub(super) fn conan_registry_url_with_base(base_url: &str, name: &str) -> String {
    let base_url = trim_end_slash(base_url);
    format!("{base_url}/v2/conans/search?q={}/*", encode_component(name))
}

pub(super) fn cpan_registry_url(name: &str) -> String {
    cpan_registry_url_with_base("https://fastapi.metacpan.org/v1", name)
}

pub(super) fn cpan_registry_url_with_base(base_url: &str, name: &str) -> String {
    let base_url = trim_end_slash(base_url);
    format!("{base_url}/download_url/{}", encode_component(name))
}

pub(super) fn cpp_registry_url(name: &str) -> String {
    cpp_registry_url_with_base(
        "https://raw.githubusercontent.com/xmake-io/xmake-repo/master",
        name,
    )
}

pub(super) fn cpp_registry_url_with_base(base_url: &str, name: &str) -> String {
    let base_url = trim_end_slash(base_url);
    let mut chars = name.chars();
    let prefix = chars.next().unwrap_or('_').to_ascii_lowercase();
    format!(
        "{base_url}/packages/{prefix}/{}/xmake.lua",
        encode_component(name)
    )
}

pub(super) fn cran_registry_url(_: &str) -> String {
    "https://cloud.r-project.org/src/contrib/PACKAGES.gz".to_owned()
}

pub(super) fn cran_registry_url_with_base(base_url: &str, _: &str) -> String {
    let base_url = trim_end_slash(base_url);
    if base_url.ends_with("/PACKAGES")
        || base_url.ends_with("/PACKAGES.gz")
        || base_url.ends_with("/PACKAGES.rds")
    {
        return base_url.to_owned();
    }
    format!("{base_url}/src/contrib/PACKAGES")
}

pub(super) fn deno_registry_url(name: &str) -> String {
    format!("https://jsr.io/{name}/meta.json")
}

pub(super) fn deno_registry_url_with_base(base_url: &str, name: &str) -> String {
    format!("{}/{name}/meta.json", trim_end_slash(base_url))
}

pub(super) fn dub_registry_url(name: &str) -> String {
    format!(
        "https://code.dlang.org/api/packages/{}/info?minimize=true",
        encode_component(name)
    )
}

pub(super) fn dub_registry_url_with_base(base_url: &str, name: &str) -> String {
    format!(
        "{}/{}/info?minimize=true",
        trim_end_slash(base_url),
        encode_component(name)
    )
}

pub(crate) fn github_registry_url(name: &str) -> String {
    GithubRepository::parse(name).map_or_else(String::new, |repository| repository.tags_url())
}

pub(crate) fn github_registry_url_with_base(base: &str, name: &str) -> String {
    GithubRepository::parse(name)
        .map_or_else(String::new, |repository| repository.api_url(base, "/tags"))
}

const STACKAGE_SNAPSHOTS_URL: &str = "https://www.stackage.org/api/v1/snapshots";

pub(super) fn hackage_registry_url(name: &str) -> String {
    if stackage_snapshot_name(name) {
        return STACKAGE_SNAPSHOTS_URL.to_owned();
    }

    format!(
        "https://hackage.haskell.org/package/{}.json",
        encode_component(name)
    )
}

pub(super) fn hackage_registry_url_with_base(base_url: &str, name: &str) -> String {
    if stackage_snapshot_name(name) {
        return hackage_registry_url(name);
    }

    let base_url = trim_end_slash(base_url);
    if base_url.ends_with("/package") {
        return format!("{base_url}/{}.json", encode_component(name));
    }

    format!("{base_url}/package/{}.json", encode_component(name))
}

fn stackage_snapshot_name(name: &str) -> bool {
    matches!(name, "stackage-lts" | "stackage-nightly")
}

pub(super) fn haxelib_registry_url(name: &str) -> String {
    haxelib_registry_url_with_base("https://lib.haxe.org", name)
}

pub(super) fn haxelib_registry_url_with_base(base_url: &str, name: &str) -> String {
    let base_url = trim_end_slash(base_url);
    format!("{base_url}/p/{}/versions/", encode_component(name))
}

pub(super) fn helm_registry_url(name: &str) -> String {
    helm_registry_url_with_base("https://charts.bitnami.com/bitnami", name)
}

pub(super) fn helm_registry_url_with_base(base_url: &str, name: &str) -> String {
    if let Some(oci) = name.strip_prefix("oci://") {
        let (registry, repository) = oci.split_once('/').unwrap_or((oci, ""));
        return format!(
            "https://{}/v2/{}/tags/list",
            trim_end_slash(registry),
            encode_oci_repository(repository)
        );
    }

    format!("{}/index.yaml", trim_end_slash(base_url))
}

fn encode_oci_repository(repository: &str) -> String {
    repository
        .split('/')
        .map(encode_component)
        .collect::<Vec<_>>()
        .join("/")
}

pub(super) fn hex_registry_url(name: &str) -> String {
    format!("https://hex.pm/api/packages/{}", encode_component(name))
}

pub(super) fn hex_registry_url_with_base(base_url: &str, name: &str) -> String {
    let base_url = trim_end_slash(base_url);
    if base_url.ends_with("/api/packages") {
        return format!("{base_url}/{}", encode_component(name));
    }
    if base_url.ends_with("/api") {
        return format!("{base_url}/packages/{}", encode_component(name));
    }

    format!("{base_url}/api/packages/{}", encode_component(name))
}

pub(super) fn julia_registry_url(name: &str) -> String {
    julia_registry_url_with_base(
        "https://raw.githubusercontent.com/JuliaRegistries/General/master",
        name,
    )
}

pub(super) fn julia_registry_url_with_base(base_url: &str, name: &str) -> String {
    let base_url = trim_end_slash(base_url);
    let prefix = name
        .chars()
        .next()
        .map(|char| char.to_uppercase().to_string())
        .unwrap_or_default();
    format!(
        "{base_url}/{}/{}/Versions.toml",
        encode_component(&prefix),
        encode_component(name)
    )
}

pub(super) fn luarocks_registry_url(name: &str) -> String {
    luarocks_registry_url_with_base("https://luarocks.org", name)
}

pub(super) fn luarocks_registry_url_with_base(base_url: &str, _: &str) -> String {
    let base_url = trim_end_slash(base_url);
    if base_url.ends_with("/manifest") {
        return base_url.to_owned();
    }
    format!("{base_url}/manifest")
}

pub(super) fn nim_registry_url(name: &str) -> String {
    if name.contains('/') {
        return github_registry_url(name);
    }
    nim_registry_url_with_base(
        "https://raw.githubusercontent.com/nim-lang/packages/master/packages.json",
        name,
    )
}

pub(super) fn nim_registry_url_with_base(base_url: &str, name: &str) -> String {
    let base_url = trim_end_slash(base_url);
    if std::path::Path::new(base_url)
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
    {
        return base_url.to_owned();
    }
    format!("{base_url}/{}", encode_component(name))
}

pub(super) fn nix_registry_url(name: &str) -> String {
    let identity = if name.contains('/') {
        name.to_owned()
    } else {
        format!("NixOS/{name}")
    };
    github_registry_url(&identity)
}

pub(super) fn nix_registry_url_with_base(base_url: &str, name: &str) -> String {
    if base_url.starts_with("https://api.github.com/repos") {
        return nix_registry_url(name);
    }
    nix_registry_url(name)
}

pub(super) fn npm_registry_url(name: &str) -> String {
    github_tags_url(name)
        .unwrap_or_else(|| format!("https://registry.npmjs.org/{}", npm_package_path(name)))
}

pub(super) fn npm_registry_url_with_base(base_url: &str, name: &str) -> String {
    format!("{}/{}", trim_end_slash(base_url), npm_package_path(name))
}

fn npm_package_path(name: &str) -> String {
    name.replace('/', "%2f")
}

pub(super) fn opam_registry_url(name: &str) -> String {
    format!(
        "https://opam.ocaml.org/packages/{}/",
        encode_component(name)
    )
}

pub(super) fn opam_registry_url_with_base(base_url: &str, name: &str) -> String {
    let base_url = trim_end_slash(base_url);
    if base_url.ends_with("/packages") {
        return format!("{base_url}/{}/", encode_component(name));
    }

    format!("{base_url}/packages/{}/", encode_component(name))
}

pub(super) fn pub_registry_url(name: &str) -> String {
    format!("https://pub.dev/api/packages/{name}")
}

pub(super) fn pub_registry_url_with_base(base_url: &str, name: &str) -> String {
    let base_url = trim_end_slash(base_url);
    if base_url.ends_with("/api/packages") {
        return format!("{base_url}/{name}");
    }

    format!("{base_url}/api/packages/{name}")
}

pub(super) fn python_registry_url(name: &str) -> String {
    format!("https://pypi.org/rss/project/{name}/releases.xml")
}

pub(super) fn python_registry_url_with_base(base_url: &str, name: &str) -> String {
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

pub(super) fn ruby_registry_url(name: &str) -> String {
    github_tags_url(name)
        .unwrap_or_else(|| format!("https://rubygems.org/api/v1/versions/{name}.json"))
}

pub(super) fn ruby_registry_url_with_base(base_url: &str, name: &str) -> String {
    let base_url = base_url.trim_end_matches('/');
    if std::path::Path::new(base_url)
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
    {
        return base_url.to_owned();
    }
    if base_url.ends_with("/api/v1/versions") {
        return format!("{base_url}/{name}.json");
    }
    format!("{base_url}/api/v1/versions/{name}.json")
}

pub(super) fn swift_registry_url(name: &str) -> String {
    swift_registry_url_with_base("https://packages.swift.org", name)
}

pub(super) fn swift_registry_url_with_base(base_url: &str, name: &str) -> String {
    let base_url = trim_end_slash(base_url);
    let (scope, package) = swift_registry_identifier(name);
    format!(
        "{base_url}/{}/{}",
        encode_component(scope),
        encode_component(package)
    )
}

fn swift_registry_identifier(name: &str) -> (&str, &str) {
    name.split_once('.')
        .or_else(|| name.split_once('/'))
        .unwrap_or(("", name))
}

pub(super) fn terraform_registry_url(name: &str) -> String {
    terraform_registry_url_with_base("https://registry.terraform.io", name)
}

pub(super) fn terraform_registry_url_with_base(base_url: &str, name: &str) -> String {
    let base_url = trim_end_slash(base_url);
    let (host, namespace, provider) = terraform_source_parts(name);
    let registry_base = if host == "registry.terraform.io" {
        base_url.to_owned()
    } else {
        format!("https://{host}")
    };
    format!(
        "{registry_base}/v1/providers/{}/{}/versions",
        encode_component(namespace),
        encode_component(provider)
    )
}

fn terraform_source_parts(name: &str) -> (&str, &str, &str) {
    let parts = name.split('/').collect::<Vec<_>>();
    match parts.as_slice() {
        [namespace, provider] => ("registry.terraform.io", namespace, provider),
        [host, namespace, provider] => (host, namespace, provider),
        _ => ("registry.terraform.io", "hashicorp", name),
    }
}

pub(super) fn unity_registry_url(name: &str) -> String {
    unity_registry_url_with_base("https://packages.unity.com", name)
}

pub(super) fn unity_registry_url_with_base(base_url: &str, name: &str) -> String {
    format!("{}/{}", trim_end_slash(base_url), name)
}

pub(super) fn vcpkg_registry_url(name: &str) -> String {
    vcpkg_registry_url_with_base(
        "https://raw.githubusercontent.com/microsoft/vcpkg/master",
        name,
    )
}

pub(super) fn vcpkg_registry_url_with_base(base_url: &str, name: &str) -> String {
    let base_url = trim_end_slash(base_url);
    let directory = versions_directory(name);
    format!("{base_url}/versions/{directory}/{name}.json")
}

fn versions_directory(name: &str) -> String {
    let first = name.chars().next().unwrap_or('_').to_ascii_lowercase();
    format!("{first}-")
}

pub(super) fn zig_registry_url(name: &str) -> String {
    if name.contains('/') {
        return github_registry_url(name);
    }
    zig_registry_url_with_base("https://pkg.ziglang.org", name)
}

pub(super) fn zig_registry_url_with_base(base_url: &str, name: &str) -> String {
    let base_url = trim_end_slash(base_url);
    format!("{base_url}/{}", encode_component(name))
}
