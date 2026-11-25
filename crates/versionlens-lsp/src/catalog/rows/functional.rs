//! Ruby, Dart, BEAM, Julia, R, Perl, OCaml, and Haskell manifests.

use crate::catalog::{FileType, row};
use crate::extract::Rule::{Keys, Lines, Record, Via};
use crate::flatten::Format::{Json, Text, Toml, Yaml};

pub(super) static ROWS: &[FileType] = &[
    row(
        "gemfile",
        &["Gemfile", "gems.rb", "*.gemspec"],
        Text,
        "rubygems",
        &[Lines(
            r#"(?m)^[ \t]*(?:gem|\w+\.add_(?:runtime_|development_)?dependency)[ \t(]+["'](?P<name>[^"']+)["'][ \t]*,[ \t]*["'](?P<version>[^"']+)["']"#,
        )],
    ),
    row(
        "podfile",
        &["Podfile", "*.podspec"],
        Text,
        "cocoapods",
        &[
            Lines(
                r#"(?m)^[ \t]*pod[ \t]+["'](?P<name>[^"']+)["'][ \t]*,[ \t]*["'](?P<version>[^"']+)["']"#,
            ),
            Lines(
                r#"(?m)^[ \t]*\w+\.dependency[ \t]+["'](?P<name>[^"']+)["'][ \t]*,[ \t]*["'](?P<version>[^"']+)["']"#,
            ),
        ],
    ),
    row(
        "pubspec",
        &["pubspec.yaml", "pubspec.yml", "pubspec_overrides.yaml"],
        Yaml,
        "pub",
        &[
            Keys("dependencies"),
            Keys("dev_dependencies"),
            Keys("dependency_overrides"),
        ],
    ),
    row(
        "mix",
        &["mix.exs"],
        Text,
        "hex",
        &[Lines(r#"\{:(?P<name>\w+),[ \t]*"(?P<version>[^"]+)""#)],
    ),
    row(
        "rebar",
        &["rebar.config"],
        Text,
        "hex",
        &[Lines(r#"\{(?P<name>\w+),[ \t]*"(?P<version>[^"]+)"\}"#)],
    ),
    row(
        "gleam",
        &["gleam.toml"],
        Toml,
        "hex",
        &[Keys("dependencies"), Keys("dev-dependencies")],
    ),
    row(
        "julia",
        &[
            "Project.toml",
            "JuliaProject.toml",
            "Manifest.toml",
            "Manifest-v*.toml",
        ],
        Toml,
        "julia",
        &[
            Keys("compat"),
            // A manifest pins each registered package; standard libraries lack `git-tree-sha1`.
            Lines(
                r#"(?m)^\[\[(?:deps\.)?(?P<name>[\w.-]+)\]\]\r?\n(?:[^\[\n][^\n]*\n)*?git-tree-sha1[^\n]*\n(?:[^\[\n][^\n]*\n)*?version[ \t]*=[ \t]*"(?P<version>[^"]+)""#,
            ),
        ],
    ),
    row(
        "renv",
        &["renv.lock"],
        Json,
        "cran",
        &[Record("Packages.*", "Package", "Version", "")],
    ),
    row(
        "r",
        &["DESCRIPTION"],
        Text,
        "cran",
        &[Lines(
            r"(?P<name>[A-Za-z][\w.]*)[ \t]*\((?P<version>[<>=]+[ \t]*[\d.-]+)\)",
        )],
    ),
    row(
        "cpanfile",
        &["cpanfile"],
        Text,
        "cpan",
        &[Lines(
            r#"(?m)^[ \t]*(?:requires|recommends|suggests)[ \t]+['"](?P<name>[\w:]+)['"][ \t]*(?:,|=>)[ \t]*['"]?(?P<version>[\w.<>= ]+?)['"]?[ \t]*;"#,
        )],
    ),
    row(
        "opam",
        &["opam", "*.opam"],
        Text,
        "opam",
        &[Lines(
            r#""(?P<name>[\w.+-]+)"[ \t]*\{[ \t]*(?:>=|=|>)[ \t]*"(?P<version>[^"]+)""#,
        )],
    ),
    row(
        "dune",
        &["dune-project"],
        Text,
        "opam",
        &[Lines(
            r"\((?P<name>[\w-]+)[ \t]+\((?P<version>(?:>=|=)[ \t]*[\w.~+-]+)\)",
        )],
    ),
    row(
        "cabal",
        &["*.cabal", "cabal.project", "stack.yaml", "stack.yml"],
        Text,
        "hackage",
        &[
            Lines(
                r"(?m)(?:build-depends:|,|^)[ \t]*(?P<name>[A-Za-z][\w-]*)[ \t]+(?P<version>(?:\^>=|>=|==)[ \t]*\d[\d.]*)",
            ),
            Via(
                "ghc",
                &Lines(r"(?m)^[ \t]*with-compiler:[ \t]*(?P<name>ghc)-(?P<version>\d[\d.]*)"),
            ),
            // Stack's `extra-deps` entries read `- package-1.2.3`.
            Lines(
                r"(?m)^[ \t]*-[ \t]+(?P<name>[A-Za-z][\w-]*?)-(?P<version>\d+(?:\.\d+)*)(?:@\S+)?[ \t]*(?:#.*)?\r?$",
            ),
        ],
    ),
    row(
        "rockspec",
        &["*.rockspec"],
        Text,
        "luarocks",
        &[Lines(
            r#"["'](?P<name>[A-Za-z][\w.-]*)[ \t]*(?P<version>(?:>=|<=|==|~>|>|<|~=)[ \t]*\d[\w.-]*)["']"#,
        )],
    ),
    row(
        "flake",
        &["flake.nix"],
        Text,
        "github",
        &[
            Lines(r"github:(?P<name>[\w.-]+/[\w.-]+)/(?P<version>v?\d[\w.-]*)"),
            Lines(r"github:(?P<name>[\w.-]+/[\w.-]+)\?ref=(?:refs/tags/)?(?P<version>v?\d[\w.-]*)"),
        ],
    ),
];
