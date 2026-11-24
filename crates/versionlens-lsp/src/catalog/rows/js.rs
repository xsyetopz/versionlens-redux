//! JavaScript, web, and other package-manager ecosystems.

use super::shared::{NPM_DEPS, TOOL_VERSION};
use crate::catalog::{FileType, row};
use crate::extract::Rule::{Keys, Lines, Spec};
use crate::flatten::Format::{Json, Text, Yaml};

pub(super) static ROWS: &[FileType] = &[
    row(
        "npm",
        &["package.json", "package.json5"],
        Json,
        "npm",
        NPM_DEPS,
    ),
    row(
        "node",
        &[".nvmrc", ".node-version"],
        Text,
        "npm",
        &[Lines(TOOL_VERSION)],
    ),
    row(
        "bun",
        &[".bun-version"],
        Text,
        "npm",
        &[Lines(TOOL_VERSION)],
    ),
    row(
        "pnpm",
        &[
            "pnpm-workspace.yaml",
            "pnpm-workspace.yml",
            "package.yaml",
            "package.yml",
            ".yarnrc.yml",
            ".yarnrc.yaml",
        ],
        Yaml,
        "npm",
        NPM_DEPS,
    ),
    row(
        "deno",
        &[
            "deno.json",
            "deno.jsonc",
            "import_map.json",
            "jsr.json",
            "jsr.jsonc",
        ],
        Json,
        "jsr",
        &[
            Spec(
                "imports.*",
                r"^(?P<registry>jsr|npm):(?P<name>@?[^@]+)@(?P<version>.+)$",
            ),
            Spec(
                "scopes.*.*",
                r"^(?P<registry>jsr|npm):(?P<name>@?[^@]+)@(?P<version>.+)$",
            ),
        ],
    ),
    row(
        "unity",
        &["Packages/manifest.json"],
        Json,
        "unity",
        &[Keys("dependencies")],
    ),
    row(
        "composer",
        &["composer.json"],
        Json,
        "composer",
        &[Keys("require"), Keys("require-dev")],
    ),
    row(
        "haxelib",
        &["haxelib.json"],
        Json,
        "haxelib",
        &[Keys("dependencies")],
    ),
    FileType {
        insert: ":",
        ..row(
            "hxml",
            &["*.hxml"],
            Text,
            "haxelib",
            &[Lines(
                r"(?m)^[ \t]*-(?:-library|lib|L)[ \t]+(?P<name>[\w.-]+)(?::(?P<version>[\w.+-]+))?[ \t]*\r?$",
            )],
        )
    },
    row(
        "dub",
        &["dub.json", "dub.selections.json"],
        Json,
        "dub",
        &[Keys("dependencies"), Keys("versions")],
    ),
    row(
        "dub-sdl",
        &["dub.sdl"],
        Text,
        "dub",
        &[Lines(
            r#"dependency[ \t]+"(?P<name>[^"]+)"[ \t]+version="(?P<version>[^"]+)""#,
        )],
    ),
    row(
        "elm",
        &["elm.json"],
        Json,
        "elm",
        &[
            Keys("dependencies.direct"),
            Keys("dependencies.indirect"),
            Keys("test-dependencies.direct"),
            Keys("test-dependencies.indirect"),
        ],
    ),
];
