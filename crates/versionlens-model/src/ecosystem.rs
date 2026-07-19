use serde::{Deserialize, Serialize};

use crate::Ecosystem::{
    AnsibleGalaxy, Bazel, Cargo, CocoaPods, Composer, Conan, Cpan, Cpp, Cran, Deno, Docker, Dotnet,
    Dub, Go, Hackage, Haxelib, Helm, Hex, Julia, LuaRocks, Maven, Nim, Nix, Npm, Opam, Pub, Python,
    Ruby, Swift, Terraform, Unity, Vcpkg, Zig,
};

#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Ecosystem {
    Cargo,
    Composer,
    Deno,
    Dotnet,
    Docker,
    Dub,
    Go,
    Maven,
    Npm,
    Python,
    Pub,
    Ruby,
    Hex,
    Opam,
    Hackage,
    Julia,
    Cran,
    Conan,
    Vcpkg,
    Swift,
    Zig,
    Nim,
    LuaRocks,
    Cpan,
    Haxelib,
    Terraform,
    Helm,
    AnsibleGalaxy,
    Bazel,
    Nix,
    Unity,
    CocoaPods,
    Cpp,
}

const ECOSYSTEM_CONFIG_NAMES: &[(&str, Ecosystem)] = &[
    ("cargo", Cargo),
    ("composer", Composer),
    ("deno", Deno),
    ("dotnet", Dotnet),
    ("docker", Docker),
    ("kustomize", Docker),
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
    ("swift", Swift),
    ("zig", Zig),
    ("nim", Nim),
    ("luarocks", LuaRocks),
    ("lua", LuaRocks),
    ("cpan", Cpan),
    ("perl", Cpan),
    ("haxelib", Haxelib),
    ("haxe", Haxelib),
    ("terraform", Terraform),
    ("opentofu", Terraform),
    ("tofu", Terraform),
    ("helm", Helm),
    ("ansible", AnsibleGalaxy),
    ("ansible-galaxy", AnsibleGalaxy),
    ("galaxy", AnsibleGalaxy),
    ("bazel", Bazel),
    ("bzlmod", Bazel),
    ("nix", Nix),
    ("unity", Unity),
    ("cocoapods", CocoaPods),
    ("cpp", Cpp),
    ("c-cpp", Cpp),
    ("c++", Cpp),
    ("cmake", Cpp),
    ("xmake", Cpp),
    ("meson", Cpp),
    ("bun", Npm),
    ("npm", Npm),
    ("pnpm", Npm),
    ("pypi", Python),
    ("python", Python),
    ("pub", Pub),
    ("ruby", Ruby),
];

const ECOSYSTEM_CONFIG_NAMESPACES: &[&str] = &[
    "cargo",
    "composer",
    "deno",
    "dotnet",
    "docker",
    "dub",
    "golang",
    "maven",
    "npm",
    "pypi",
    "pub",
    "ruby",
    "hex",
    "opam",
    "hackage",
    "julia",
    "cran",
    "conan",
    "vcpkg",
    "swift",
    "zig",
    "nim",
    "luarocks",
    "cpan",
    "haxelib",
    "terraform",
    "helm",
    "ansible",
    "bazel",
    "nix",
    "unity",
    "cocoapods",
    "cpp",
];

pub fn ecosystem_from_config_name(name: &str) -> Option<Ecosystem> {
    ECOSYSTEM_CONFIG_NAMES
        .iter()
        .find_map(|(candidate, ecosystem)| (*candidate == name).then_some(*ecosystem))
}

pub fn ecosystem_config_namespace(ecosystem: Ecosystem) -> &'static str {
    ECOSYSTEM_CONFIG_NAMESPACES[ecosystem as usize]
}

#[cfg(test)]
#[path = "ecosystem/tests.rs"]
mod tests;
