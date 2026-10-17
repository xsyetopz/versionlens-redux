use serde::{Deserialize, Serialize};

#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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
    GitHub,
}

/// One row per ecosystem, in `Ecosystem` declaration order, so a row is found
/// by `ecosystem as usize`.
struct EcosystemRow {
    ecosystem: Ecosystem,
    /// Settings namespace, such as `golang` in `versionlens.golang.*`.
    namespace: &'static str,
    /// Provider ID used by the registry layer.
    provider_id: &'static str,
    /// Names accepted in settings, including legacy names.
    config_names: &'static [&'static str],
}

const ECOSYSTEMS: &[EcosystemRow] = &[
    EcosystemRow {
        ecosystem: Ecosystem::Cargo,
        namespace: "cargo",
        provider_id: "cargo",
        config_names: &["cargo"],
    },
    EcosystemRow {
        ecosystem: Ecosystem::Composer,
        namespace: "composer",
        provider_id: "composer",
        config_names: &["composer"],
    },
    EcosystemRow {
        ecosystem: Ecosystem::Deno,
        namespace: "deno",
        provider_id: "deno",
        config_names: &["deno"],
    },
    EcosystemRow {
        ecosystem: Ecosystem::Dotnet,
        namespace: "dotnet",
        provider_id: "dotnet",
        config_names: &["dotnet"],
    },
    EcosystemRow {
        ecosystem: Ecosystem::Docker,
        namespace: "docker",
        provider_id: "docker",
        config_names: &["docker", "kustomize"],
    },
    EcosystemRow {
        ecosystem: Ecosystem::Dub,
        namespace: "dub",
        provider_id: "dub",
        config_names: &["dub"],
    },
    EcosystemRow {
        ecosystem: Ecosystem::Go,
        namespace: "golang",
        provider_id: "go",
        config_names: &["go", "golang"],
    },
    EcosystemRow {
        ecosystem: Ecosystem::Maven,
        namespace: "maven",
        provider_id: "maven",
        config_names: &["maven"],
    },
    EcosystemRow {
        ecosystem: Ecosystem::Npm,
        namespace: "npm",
        provider_id: "npm",
        config_names: &["bun", "npm", "pnpm"],
    },
    EcosystemRow {
        ecosystem: Ecosystem::Python,
        namespace: "pypi",
        provider_id: "python",
        config_names: &["pypi", "python"],
    },
    EcosystemRow {
        ecosystem: Ecosystem::Pub,
        namespace: "pub",
        provider_id: "pub",
        config_names: &["pub"],
    },
    EcosystemRow {
        ecosystem: Ecosystem::Ruby,
        namespace: "ruby",
        provider_id: "ruby",
        config_names: &["ruby"],
    },
    EcosystemRow {
        ecosystem: Ecosystem::Hex,
        namespace: "hex",
        provider_id: "hex",
        config_names: &["hex", "beam"],
    },
    EcosystemRow {
        ecosystem: Ecosystem::Opam,
        namespace: "opam",
        provider_id: "opam",
        config_names: &["opam", "ocaml"],
    },
    EcosystemRow {
        ecosystem: Ecosystem::Hackage,
        namespace: "hackage",
        provider_id: "hackage",
        config_names: &["hackage", "haskell"],
    },
    EcosystemRow {
        ecosystem: Ecosystem::Julia,
        namespace: "julia",
        provider_id: "julia",
        config_names: &["julia"],
    },
    EcosystemRow {
        ecosystem: Ecosystem::Cran,
        namespace: "cran",
        provider_id: "cran",
        config_names: &["cran", "r"],
    },
    EcosystemRow {
        ecosystem: Ecosystem::Conan,
        namespace: "conan",
        provider_id: "conan",
        config_names: &["conan"],
    },
    EcosystemRow {
        ecosystem: Ecosystem::Vcpkg,
        namespace: "vcpkg",
        provider_id: "vcpkg",
        config_names: &["vcpkg"],
    },
    EcosystemRow {
        ecosystem: Ecosystem::Swift,
        namespace: "swift",
        provider_id: "swift",
        config_names: &["swift"],
    },
    EcosystemRow {
        ecosystem: Ecosystem::Zig,
        namespace: "zig",
        provider_id: "zig",
        config_names: &["zig"],
    },
    EcosystemRow {
        ecosystem: Ecosystem::Nim,
        namespace: "nim",
        provider_id: "nim",
        config_names: &["nim"],
    },
    EcosystemRow {
        ecosystem: Ecosystem::LuaRocks,
        namespace: "luarocks",
        provider_id: "luarocks",
        config_names: &["luarocks", "lua"],
    },
    EcosystemRow {
        ecosystem: Ecosystem::Cpan,
        namespace: "cpan",
        provider_id: "cpan",
        config_names: &["cpan", "perl"],
    },
    EcosystemRow {
        ecosystem: Ecosystem::Haxelib,
        namespace: "haxelib",
        provider_id: "haxelib",
        config_names: &["haxelib", "haxe"],
    },
    EcosystemRow {
        ecosystem: Ecosystem::Terraform,
        namespace: "terraform",
        provider_id: "terraform",
        config_names: &["terraform", "opentofu", "tofu"],
    },
    EcosystemRow {
        ecosystem: Ecosystem::Helm,
        namespace: "helm",
        provider_id: "helm",
        config_names: &["helm"],
    },
    EcosystemRow {
        ecosystem: Ecosystem::AnsibleGalaxy,
        namespace: "ansible",
        provider_id: "ansible",
        config_names: &["ansible", "ansible-galaxy", "galaxy"],
    },
    EcosystemRow {
        ecosystem: Ecosystem::Bazel,
        namespace: "bazel",
        provider_id: "bazel",
        config_names: &["bazel", "bzlmod"],
    },
    EcosystemRow {
        ecosystem: Ecosystem::Nix,
        namespace: "nix",
        provider_id: "nix",
        config_names: &["nix"],
    },
    EcosystemRow {
        ecosystem: Ecosystem::Unity,
        namespace: "unity",
        provider_id: "unity",
        config_names: &["unity"],
    },
    EcosystemRow {
        ecosystem: Ecosystem::CocoaPods,
        namespace: "cocoapods",
        provider_id: "cocoapods",
        config_names: &["cocoapods"],
    },
    EcosystemRow {
        ecosystem: Ecosystem::Cpp,
        namespace: "cpp",
        provider_id: "cpp",
        config_names: &["cpp", "c-cpp", "c++", "cmake", "xmake", "meson"],
    },
    EcosystemRow {
        ecosystem: Ecosystem::GitHub,
        namespace: "github",
        provider_id: "github",
        config_names: &["github"],
    },
];

fn row(ecosystem: Ecosystem) -> &'static EcosystemRow {
    &ECOSYSTEMS[ecosystem as usize]
}

pub fn all_ecosystems() -> impl Iterator<Item = Ecosystem> {
    ECOSYSTEMS.iter().map(|row| row.ecosystem)
}

pub fn ecosystem_from_config_name(name: &str) -> Option<Ecosystem> {
    ECOSYSTEMS
        .iter()
        .find(|row| row.config_names.contains(&name))
        .map(|row| row.ecosystem)
}

pub fn ecosystem_config_namespace(ecosystem: Ecosystem) -> &'static str {
    row(ecosystem).namespace
}

pub fn ecosystem_provider_id(ecosystem: Ecosystem) -> &'static str {
    row(ecosystem).provider_id
}

#[cfg(test)]
#[path = "ecosystem/tests.rs"]
mod tests;
