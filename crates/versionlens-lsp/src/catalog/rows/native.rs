//! Systems and native toolchains.

use super::shared::{GITHUB_ARCHIVE, GITHUB_FRAGMENT, GITHUB_RELEASE};
use crate::catalog::{FileType, row};
use crate::extract::Rule::{Keys, Lines, Record, Via};
use crate::flatten::Format::{Json, Text, Toml, Yaml};

pub(super) static ROWS: &[FileType] = &[
    row(
        "cargo",
        &["Cargo.toml"],
        Toml,
        "cargo",
        &[
            Keys("dependencies"),
            Keys("dev-dependencies"),
            Keys("build-dependencies"),
            Keys("target.*.dependencies"),
            Keys("target.*.dev-dependencies"),
            Keys("target.*.build-dependencies"),
            Keys("workspace.dependencies"),
        ],
    ),
    row(
        "go",
        &["go.mod", "go.work"],
        Text,
        "go",
        &[
            Lines(
                r"(?m)^(?:require[ \t]+|[ \t]+)(?P<name>[\w.~-]+\.[\w.~/-]+)[ \t]+(?P<version>v\d[^\s]*)",
            ),
            // The `go` and `toolchain` directives pin the Go release itself.
            Via(
                "golang",
                &Lines(r"(?m)^(?:go[ \t]+|toolchain[ \t]+go)(?P<version>\d[\w.]*)[ \t]*\r?$"),
            ),
        ],
    ),
    // dep and Glide, the Go package managers before modules.
    row(
        "gopkg",
        &["Gopkg.toml"],
        Toml,
        "go",
        &[
            Record("constraint.*", "name", "version", ""),
            Record("override.*", "name", "version", ""),
        ],
    ),
    row(
        "glide",
        &["glide.yaml", "glide.yml"],
        Yaml,
        "go",
        &[
            Record("import.*", "package", "version", ""),
            Record("testImport.*", "package", "version", ""),
        ],
    ),
    row(
        "bazel",
        &["MODULE.bazel"],
        Text,
        "bazel",
        &[Lines(
            r#"bazel_dep\(\s*name\s*=\s*"(?P<name>[^"]+)"\s*,\s*version\s*=\s*"(?P<version>[^"]+)""#,
        )],
    ),
    row(
        "vcpkg",
        &["vcpkg.json"],
        Json,
        "vcpkg",
        &[
            Record("dependencies.*", "name", "version>=", ""),
            Record(
                "overrides.*",
                "name",
                "version|version-semver|version-string|version-date",
                "",
            ),
        ],
    ),
    row(
        "conan",
        &["conanfile.txt", "conanfile.py"],
        Text,
        "conan",
        &[Lines(
            r#"(?m)(?:^[ \t]*|requires\(\s*["']|["'])(?P<name>[a-z0-9][\w.+-]*)/(?P<version>\d[\w.+-]*)"#,
        )],
    ),
    row(
        "shards",
        &["shard.yml"],
        Yaml,
        "github",
        &[
            Record("dependencies.*", "github", "version", ""),
            Record("development_dependencies.*", "github", "version", ""),
        ],
    ),
    row(
        "mint",
        &["Mintfile"],
        Text,
        "github",
        &[Lines(
            r"(?m)^[ \t]*(?:https://github\.com/)?(?P<name>[\w.-]+/[\w.-]+?)(?:\.git)?@(?P<version>v?\d[\w.-]*)",
        )],
    ),
    row(
        "carthage",
        &["Cartfile", "Cartfile.private"],
        Text,
        "github",
        &[Lines(
            r#"(?m)^[ \t]*github[ \t]+"(?P<name>[\w.-]+/[\w.-]+)"[ \t]+"?(?P<version>(?:~>|>=|==)?[ \t]*v?\d[\w.-]*)"#,
        )],
    ),
    row(
        "swift",
        &["Package.swift"],
        Text,
        "swift",
        &[Lines(
            r#"\.package\(\s*url:\s*"https://github\.com/(?P<name>[\w.-]+/[\w-]+(?:\.[\w-]+)*?)(?:\.git)?"\s*,\s*(?:from:|exact:|\.upToNextMajor\(from:|\.upToNextMinor\(from:)\s*"(?P<version>[^"]+)""#,
        )],
    ),
    row(
        "cmake",
        &["CMakeLists.txt", "*.cmake"],
        Text,
        "github",
        &[
            Lines(r"gh:(?P<name>[\w.-]+/[\w.-]+)#(?P<version>v?\d[\w.-]*)"),
            Lines(
                r"GIT_REPOSITORY\s+https://github\.com/(?P<name>[\w.-]+/[\w-]+)(?:\.git)?\s+GIT_TAG\s+(?P<version>v?\d[\w.-]*)",
            ),
        ],
    ),
    row(
        "cargo-config",
        &[".cargo/config.toml", ".cargo/config"],
        Toml,
        "cargo",
        &[Keys("patch.*")],
    ),
    row(
        "rust",
        &["rust-toolchain", "rust-toolchain.toml"],
        Text,
        "rust",
        &[Lines(
            r#"(?m)^[ \t]*(?:channel[ \t]*=[ \t]*["'])?(?P<version>\d+\.\d+(?:\.\d+)?)["']?[ \t]*(?:#.*)?\r?$"#,
        )],
    ),
    row(
        "golang",
        &[".go-version"],
        Text,
        "golang",
        &[Lines(r"\A\s*(?:go)?(?P<version>\d[\w.]*)\s*\z")],
    ),
    row(
        "zig",
        &["build.zig.zon"],
        Text,
        "github",
        &[
            Lines(GITHUB_ARCHIVE),
            Lines(GITHUB_RELEASE),
            Lines(GITHUB_FRAGMENT),
        ],
    ),
    row(
        "nimble",
        &["*.nimble"],
        Text,
        "github",
        &[Lines(
            r#"requires[ \t(]+"https?://github\.com/(?P<name>[\w.-]+/[\w-]+)(?:\.git)?[ \t]+(?P<version>(?:>=|<=|==|\^=|~=|>|<)[ \t]*v?\d[\w.-]*)""#,
        )],
    ),
    row(
        "xmake",
        &["xmake.lua"],
        Text,
        "xmake",
        &[Lines(
            r#"add_requires\([^)\n]*?["'](?P<name>[A-Za-z][\w.+-]*)[ \t]+(?P<version>\d[\w.*]*)["']"#,
        )],
    ),
    row(
        "meson-wrap",
        &["subprojects/*.wrap"],
        Text,
        "github",
        &[
            Lines(
                r"(?m)^[ \t]*url[ \t]*=[ \t]*https://github\.com/(?P<name>[\w.-]+/[\w-]+(?:\.[\w-]+)*?)(?:\.git)?[ \t]*\r?\n(?:[^\n]*\n)*?[ \t]*revision[ \t]*=[ \t]*(?P<version>v?\d[\w.-]*)",
            ),
            Lines(GITHUB_ARCHIVE),
            Lines(GITHUB_RELEASE),
        ],
    ),
    row(
        "bazel-workspace",
        &["WORKSPACE", "WORKSPACE.bazel"],
        Text,
        "github",
        &[Lines(GITHUB_ARCHIVE), Lines(GITHUB_RELEASE)],
    ),
];
