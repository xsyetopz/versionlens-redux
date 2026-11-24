//! Python manifests.

use super::shared::{PEP508, PYPROJECT, TOOL_VERSION};
use crate::catalog::{FileType, row};
use crate::extract::Rule::{Keys, Lines, Spec};
use crate::flatten::Format::{Text, Toml};

pub(super) static ROWS: &[FileType] = &[
    row("pyproject", &["pyproject.toml"], Toml, "pypi", PYPROJECT),
    row(
        "pipfile",
        &["Pipfile"],
        Toml,
        "pypi",
        &[Keys("packages"), Keys("dev-packages")],
    ),
    FileType {
        insert: "==",
        ..row(
            "requirements",
            &[
                "*requirements*.txt",
                "*constraints*.txt",
                "requirements/*.txt",
                "requirements*.in",
            ],
            Text,
            "pypi",
            &[Lines(
                r"(?m)^[ \t]*(?P<name>[A-Za-z0-9][A-Za-z0-9._-]*)[ \t]*(?:\[[^\]\n]*\])?[ \t]*(?P<version>(?:===|==|~=|!=|<=|>=|<|>)[ \t]*[^;,\s#\\]+(?:[ \t]*,[ \t]*(?:===|==|~=|!=|<=|>=|<|>)[ \t]*[^;,\s#\\]+)*)?[ \t]*(?:[;#\\].*)?\r?$",
            )],
        )
    },
    row(
        "uv",
        &["uv.toml"],
        Toml,
        "pypi",
        &[
            Spec("dev-dependencies.*", PEP508),
            Spec("constraint-dependencies.*", PEP508),
            Spec("override-dependencies.*", PEP508),
        ],
    ),
    row(
        "pixi",
        &["pixi.toml"],
        Toml,
        "conda",
        &[
            Keys("dependencies"),
            Keys("build-dependencies"),
            Keys("host-dependencies"),
            Keys("feature.*.dependencies"),
            Keys("target.*.dependencies"),
        ],
    ),
    row(
        "python",
        &[".python-version"],
        Text,
        "python",
        &[Lines(TOOL_VERSION)],
    ),
];
