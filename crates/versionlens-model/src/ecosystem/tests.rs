use super::{ecosystem_config_namespace, ecosystem_from_config_name};
use crate::Ecosystem::{Cpp, Docker};

#[test]
fn maps_config_names_and_legacy_names_to_ecosystems() {
    for (name, ecosystem) in super::ECOSYSTEM_CONFIG_NAMES {
        assert_eq!(ecosystem_from_config_name(name), Some(*ecosystem));
    }
}

#[test]
fn ignores_unknown_config_names() {
    assert_eq!(ecosystem_from_config_name("unknown"), None);
}

#[test]
fn maps_ecosystems_to_config_namespaces() {
    let cases = [
        (crate::Ecosystem::Cargo, "cargo"),
        (crate::Ecosystem::Composer, "composer"),
        (crate::Ecosystem::Deno, "deno"),
        (crate::Ecosystem::Dotnet, "dotnet"),
        (Docker, "docker"),
        (crate::Ecosystem::Dub, "dub"),
        (crate::Ecosystem::Go, "golang"),
        (crate::Ecosystem::Maven, "maven"),
        (crate::Ecosystem::Hex, "hex"),
        (crate::Ecosystem::Opam, "opam"),
        (crate::Ecosystem::Hackage, "hackage"),
        (crate::Ecosystem::Julia, "julia"),
        (crate::Ecosystem::Cran, "cran"),
        (crate::Ecosystem::Conan, "conan"),
        (crate::Ecosystem::Vcpkg, "vcpkg"),
        (Cpp, "cpp"),
        (crate::Ecosystem::Swift, "swift"),
        (crate::Ecosystem::Zig, "zig"),
        (crate::Ecosystem::Nim, "nim"),
        (crate::Ecosystem::LuaRocks, "luarocks"),
        (crate::Ecosystem::Cpan, "cpan"),
        (crate::Ecosystem::Haxelib, "haxelib"),
        (crate::Ecosystem::Npm, "npm"),
        (crate::Ecosystem::Python, "pypi"),
        (crate::Ecosystem::Pub, "pub"),
        (crate::Ecosystem::Ruby, "ruby"),
        (crate::Ecosystem::GitHub, "github"),
    ];

    for (ecosystem, namespace) in cases {
        assert_eq!(ecosystem_config_namespace(ecosystem), namespace);
    }
}
