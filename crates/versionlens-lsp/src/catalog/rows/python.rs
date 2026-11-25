//! Python manifests.

use super::shared::{PEP508, PYPROJECT, TOOL_VERSION};
use crate::catalog::{FileType, row};
use crate::extract::Rule::{Keys, Lines, Spec, Via};
use crate::flatten::Format::{Text, Toml, Yaml};

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
        "setuptools",
        &["setup.cfg", "setup.py", "tox.ini"],
        Text,
        "pypi",
        &[
            // setup.cfg and tox.ini list requirements on indented continuation lines, setup.py in quoted
            // strings; each needs a comparison, so `packages` values and classifiers stay out.
            Lines(
                r"(?m)^[ \t]+(?P<name>[A-Za-z0-9][\w.-]*)[ \t]*(?:\[[^\]\n]*\])?[ \t]*(?P<version>(?:===|==|~=|!=|<=|>=|<|>)[ \t]*[^;,\s#]+(?:[ \t]*,[ \t]*(?:===|==|~=|!=|<=|>=|<|>)[ \t]*[^;,\s#]+)*)[ \t]*(?:[;#].*)?\r?$",
            ),
            Lines(
                r#"["'](?P<name>[A-Za-z0-9][\w.-]*)[ \t]*(?:\[[^\]"']*\])?[ \t]*(?P<version>(?:===|==|~=|!=|<=|>=|<|>)[ \t]*[^;,\s"']+(?:[ \t]*,[ \t]*(?:===|==|~=|!=|<=|>=|<|>)[ \t]*[^;,\s"']+)*)[ \t]*(?:;[^"']*)?["']"#,
            ),
        ],
    ),
    row(
        "conda",
        &["environment.yml", "environment.yaml"],
        Yaml,
        "conda",
        &[
            // A conda spec: an optional `channel::`, the name, then a version. A bare or `=` version
            // such as `=3.11` matches by prefix, as conda reads it.
            Spec(
                "dependencies.*",
                r"^\s*(?:[\w.-]+::)?(?P<name>[A-Za-z0-9][\w.-]*)(?:\s*=?(?P<version>(?:==|>=|<=|!=|~=|<|>)?\s*\d[^\s,|#]*(?:\s*,\s*(?:==|>=|<=|!=|<|>)\s*[^\s,|#]+)*))?\s*$",
            ),
            Via("pypi", &Spec("dependencies.*.pip.*", PEP508)),
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
