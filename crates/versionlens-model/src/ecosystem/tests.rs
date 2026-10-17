use super::{
    ECOSYSTEMS, all_ecosystems, ecosystem_config_namespace, ecosystem_from_config_name,
    ecosystem_provider_id,
};
use crate::Ecosystem::{Cpp, Docker};

#[test]
fn maps_config_names_and_legacy_names_to_ecosystems() {
    for row in ECOSYSTEMS {
        for name in row.config_names {
            assert_eq!(ecosystem_from_config_name(name), Some(row.ecosystem));
        }
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

#[test]
fn keeps_rows_in_ecosystem_declaration_order() {
    for (index, row) in ECOSYSTEMS.iter().enumerate() {
        assert_eq!(row.ecosystem as usize, index, "{:?}", row.ecosystem);
    }
    assert_eq!(
        all_ecosystems().count(),
        crate::Ecosystem::GitHub as usize + 1
    );
}

#[test]
fn accepts_each_config_name_for_one_ecosystem_only() {
    let mut names = ECOSYSTEMS
        .iter()
        .flat_map(|row| row.config_names.iter())
        .collect::<Vec<_>>();
    let count = names.len();
    names.sort();
    names.dedup();
    assert_eq!(names.len(), count);
}

#[test]
fn maps_ecosystems_to_provider_ids() {
    assert_eq!(ecosystem_provider_id(crate::Ecosystem::Go), "go");
    assert_eq!(ecosystem_provider_id(crate::Ecosystem::Python), "python");
    assert_eq!(ecosystem_provider_id(crate::Ecosystem::Cargo), "cargo");
}
