use serde::{Deserialize, Serialize};

#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// A package ecosystem whose dependencies `VersionLens` can check.
pub enum Ecosystem {
    /// Rust crates on crates.io.
    Cargo,
    /// PHP packages on Packagist.
    Composer,
    /// Deno modules and JSR packages.
    Deno,
    /// .NET packages on `NuGet`.
    Dotnet,
    /// Container images in Docker registries.
    Docker,
    /// D packages on the DUB registry.
    Dub,
    /// Go modules.
    Go,
    /// JVM artifacts on Maven repositories.
    Maven,
    /// JavaScript packages on npm-compatible registries.
    Npm,
    /// Python packages on `PyPI`.
    Python,
    /// Dart and Flutter packages on pub.dev.
    Pub,
    /// Ruby gems on `RubyGems`.
    Ruby,
    /// Elixir and Erlang packages on Hex.
    Hex,
    /// OCaml packages on opam.
    Opam,
    /// Haskell packages on Hackage.
    Hackage,
    /// Julia packages.
    Julia,
    /// R packages on CRAN.
    Cran,
    /// C and C++ packages on Conan.
    Conan,
    /// C and C++ packages in vcpkg.
    Vcpkg,
    /// Swift packages.
    Swift,
    /// Zig packages.
    Zig,
    /// Nim packages.
    Nim,
    /// Lua packages on `LuaRocks`.
    LuaRocks,
    /// Perl modules on CPAN.
    Cpan,
    /// Haxe libraries on Haxelib.
    Haxelib,
    /// Terraform and `OpenTofu` providers and modules.
    Terraform,
    /// Helm charts.
    Helm,
    /// Ansible roles and collections on Galaxy.
    AnsibleGalaxy,
    /// Bazel modules.
    Bazel,
    /// Nix flake inputs.
    Nix,
    /// Unity packages.
    Unity,
    /// `CocoaPods` pods.
    CocoaPods,
    /// C and C++ dependencies declared in `CMake`, xmake, and Meson files.
    Cpp,
    /// GitHub Actions references and repository tags.
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

/// Iterate over every ecosystem in declaration order.
pub fn all_ecosystems() -> impl Iterator<Item = Ecosystem> {
    ECOSYSTEMS.iter().map(|row| row.ecosystem)
}

/// Find the ecosystem that accepts `name` as a settings name, including legacy names.
pub fn ecosystem_from_config_name(name: &str) -> Option<Ecosystem> {
    ECOSYSTEMS
        .iter()
        .find(|row| row.config_names.contains(&name))
        .map(|row| row.ecosystem)
}

/// Return the settings namespace of `ecosystem`, such as `golang` for Go.
pub fn ecosystem_config_namespace(ecosystem: Ecosystem) -> &'static str {
    row(ecosystem).namespace
}

/// Return the registry-layer provider ID of `ecosystem`.
pub fn ecosystem_provider_id(ecosystem: Ecosystem) -> &'static str {
    row(ecosystem).provider_id
}

#[cfg(test)]
mod tests {
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
}
