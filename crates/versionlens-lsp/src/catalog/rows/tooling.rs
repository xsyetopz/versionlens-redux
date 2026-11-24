//! Runtime-version files, tool managers, and repository tooling configuration.
//!
//! Tool managers name runtimes by registry id (`python`, `ruby`, `rust`, `golang`) or through
//! `registry::alias` (`terraform`). The nested `registry` and `name` groups capture one word twice,
//! so the dependency is looked up where that word says and keeps it as its name.

use super::shared::{DOCKER, TOOL_VERSION};
use crate::catalog::{FileType, row};
use crate::extract::Rule::{Lines, Spec};
use crate::flatten::Format::{Json, Text, Yaml};

pub(super) static ROWS: &[FileType] = &[
    row(
        "tool-versions",
        &[".tool-versions"],
        Text,
        "npm",
        &[
            Lines(r"(?m)^[ \t]*(?P<name>node|bun|pnpm)(?:js)?[ \t]+(?P<version>\d[\w.]*)"),
            Lines(
                r"(?m)^[ \t]*(?P<registry>(?P<name>python|ruby|rust|terraform|golang))[ \t]+(?P<version>\d[\w.]*)",
            ),
        ],
    ),
    row(
        "mise",
        &[
            "mise.toml",
            ".mise.toml",
            "mise.*.toml",
            ".mise.*.toml",
            ".mise/config.toml",
            ".config/mise/config.toml",
        ],
        Text,
        "npm",
        &[
            Lines(
                r#"(?m)^[ \t]*["']?(?P<name>node|bun|pnpm|npm)["']?[ \t]*=[ \t]*(?:\{[^}\n]*?[ {,]version[ \t]*=[ \t]*)?["'](?P<version>\d[\w.]*)["']"#,
            ),
            Lines(
                r#"(?m)^[ \t]*["']?(?P<registry>(?P<name>python|ruby|rust|terraform|golang))["']?[ \t]*=[ \t]*(?:\{[^}\n]*?[ {,]version[ \t]*=[ \t]*)?["'](?P<version>\d[\w.]*)["']"#,
            ),
            Lines(
                r#"(?m)^[ \t]*["'](?P<registry>npm|cargo|pipx|go):(?P<name>[^"'\s]+)["'][ \t]*=[ \t]*(?:\{[^}\n]*?[ {,]version[ \t]*=[ \t]*)?["'](?P<version>\d[\w.]*)["']"#,
            ),
        ],
    ),
    row(
        "ruby",
        &[".ruby-version"],
        Text,
        "ruby",
        &[Lines(r"\A\s*(?:ruby-)?(?P<version>\d[\w.]*)\s*\z")],
    ),
    row(
        "terraform-cli",
        &[".terraform-version"],
        Text,
        "terraform-cli",
        &[Lines(TOOL_VERSION)],
    ),
    row(
        "pre-commit",
        &[".pre-commit-config.yaml", ".pre-commit-config.yml"],
        Yaml,
        "github",
        &[Lines(
            r#"(?m)repo:[ \t]*["']?https://github\.com/(?P<name>[\w.-]+/[\w-]+(?:\.[\w-]+)*?)(?:\.git)?["']?[ \t]*\r?\n[ \t]*rev:[ \t]*["']?(?P<version>v?\d[\w.-]*)"#,
        )],
    ),
    row(
        "devcontainer",
        &["devcontainer.json"],
        Json,
        "docker",
        &[Spec("image", DOCKER)],
    ),
];
