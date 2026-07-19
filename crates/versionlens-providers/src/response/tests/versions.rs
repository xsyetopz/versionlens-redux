use super::{
    assert_latest, latest_version_for_requirement, latest_version_from_response,
    latest_version_from_response_with_prereleases, latest_version_with_tags, npm_build_versions,
    release_versions_from_response, release_versions_from_response_for_package,
};
use versionlens_model::Ecosystem::{
    AnsibleGalaxy, Bazel, Cargo, CocoaPods, Conan, Cpan, Cpp, Cran, Deno, Docker, Dotnet, Go,
    Hackage, Hex, Julia, LuaRocks, Maven, Nim, Nix, Npm, Opam, Pub, Python, Ruby, Swift, Terraform,
    Vcpkg, Zig,
};

include!("versions/json.rs");
include!("versions/ecosystems.rs");
include!("versions/go.rs");
include!("versions/packages.rs");
include!("versions/fallbacks.rs");
