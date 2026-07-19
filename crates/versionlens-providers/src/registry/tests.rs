use super::{
    RegistryResponseKind, docker_hub_body_has_next_page, docker_hub_tags_page_url,
    dotnet_package_url_from_service_index, is_composer_platform_dependency, is_registry_dependency,
    is_registry_requirement, merge_docker_hub_response_pages, provider_id, registry_endpoint,
    registry_endpoint_with_base, registry_url, registry_url_with_base,
};
use versionlens_model::Ecosystem::{
    AnsibleGalaxy, Bazel, Cargo, CocoaPods, Composer, Conan, Cpan, Cpp, Cran, Deno, Docker, Dotnet,
    Dub, Go, Hackage, Haxelib, Helm, Hex, Julia, LuaRocks, Maven, Nim, Nix, Npm, Opam, Pub, Python,
    Ruby, Swift, Terraform, Unity, Vcpkg, Zig,
};

include!("tests/urls.rs");
include!("tests/contracts.rs");
