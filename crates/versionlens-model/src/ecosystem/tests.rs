use super::{ecosystem_config_namespace, ecosystem_from_config_name};
use crate::Ecosystem::{
    Cargo, Composer, Conan, Cpan, Cpp, Cran, Deno, Docker, Dotnet, Dub, Go, Hackage, Haxelib, Hex,
    Julia, LuaRocks, Maven, Nim, Npm, Opam, Pub, Python, Ruby, Swift, Vcpkg, Zig,
};

#[test]
fn maps_config_names_and_legacy_names_to_ecosystems() {
    let cases = [
        ("cargo", Cargo),
        ("composer", Composer),
        ("deno", Deno),
        ("dotnet", Dotnet),
        ("docker", Docker),
        ("dub", Dub),
        ("go", Go),
        ("golang", Go),
        ("maven", Maven),
        ("hex", Hex),
        ("beam", Hex),
        ("opam", Opam),
        ("ocaml", Opam),
        ("hackage", Hackage),
        ("haskell", Hackage),
        ("julia", Julia),
        ("cran", Cran),
        ("r", Cran),
        ("conan", Conan),
        ("vcpkg", Vcpkg),
        ("cpp", Cpp),
        ("c-cpp", Cpp),
        ("cmake", Cpp),
        ("xmake", Cpp),
        ("meson", Cpp),
        ("swift", Swift),
        ("zig", Zig),
        ("nim", Nim),
        ("luarocks", LuaRocks),
        ("lua", LuaRocks),
        ("cpan", Cpan),
        ("perl", Cpan),
        ("haxelib", Haxelib),
        ("haxe", Haxelib),
        ("bun", Npm),
        ("npm", Npm),
        ("pnpm", Npm),
        ("pypi", Python),
        ("python", Python),
        ("pub", Pub),
        ("ruby", Ruby),
    ];

    for (name, ecosystem) in cases {
        assert_eq!(ecosystem_from_config_name(name), Some(ecosystem));
    }
}

#[test]
fn ignores_unknown_config_names() {
    assert_eq!(ecosystem_from_config_name("unknown"), None);
}

#[test]
fn maps_ecosystems_to_config_namespaces() {
    let cases = [
        (Cargo, "cargo"),
        (Composer, "composer"),
        (Deno, "deno"),
        (Dotnet, "dotnet"),
        (Docker, "docker"),
        (Dub, "dub"),
        (Go, "golang"),
        (Maven, "maven"),
        (Hex, "hex"),
        (Opam, "opam"),
        (Hackage, "hackage"),
        (Julia, "julia"),
        (Cran, "cran"),
        (Conan, "conan"),
        (Vcpkg, "vcpkg"),
        (Cpp, "cpp"),
        (Swift, "swift"),
        (Zig, "zig"),
        (Nim, "nim"),
        (LuaRocks, "luarocks"),
        (Cpan, "cpan"),
        (Haxelib, "haxelib"),
        (Npm, "npm"),
        (Python, "pypi"),
        (Pub, "pub"),
        (Ruby, "ruby"),
    ];

    for (ecosystem, namespace) in cases {
        assert_eq!(ecosystem_config_namespace(ecosystem), namespace);
    }
}
