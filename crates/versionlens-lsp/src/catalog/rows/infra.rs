//! Containers, CI, orchestration, and infrastructure files.

use super::shared::{ACTION, DOCKER};
use crate::catalog::{FileType, row};
use crate::extract::Rule::{Keys, Lines, Record, Spec};
use crate::flatten::Format::{Text, Yaml};

pub(super) static ROWS: &[FileType] = &[
    row(
        "dockerfile",
        &[
            "Dockerfile",
            "*.Dockerfile",
            "Dockerfile.*",
            "Containerfile",
        ],
        Text,
        "docker",
        &[Lines(
            r"(?mi)^[ \t]*FROM[ \t]+(?:--\S+[ \t]+)*(?P<name>[\w./-]+(?::\d+/[\w./-]+)?):(?P<version>[\w.-]+)",
        )],
    ),
    row(
        "compose",
        &[
            "compose.yaml",
            "compose.yml",
            "compose.*.yaml",
            "compose.*.yml",
            "docker-compose*.yaml",
            "docker-compose*.yml",
            "*.compose.yaml",
            "*.compose.yml",
        ],
        Yaml,
        "docker",
        &[Spec("services.*.image", DOCKER)],
    ),
    row(
        "kustomize",
        &["kustomization.yaml", "kustomization.yml", "Kustomization"],
        Yaml,
        "docker",
        &[Record("images.*", "name", "newTag", "")],
    ),
    row(
        "github-actions",
        &[
            ".github/workflows/**/*.yml",
            ".github/workflows/**/*.yaml",
            ".github/actions/**/*.yml",
            ".github/actions/**/*.yaml",
            "action.yml",
            "action.yaml",
        ],
        Yaml,
        "github",
        &[
            Spec("jobs.*.steps.*.uses", ACTION),
            Spec("jobs.*.uses", ACTION),
            Spec("runs.steps.*.uses", ACTION),
        ],
    ),
    row(
        "helm",
        &["Chart.yaml"],
        Yaml,
        "helm",
        &[Record("dependencies.*", "name", "version", "repository")],
    ),
    row(
        "ansible",
        &["requirements.yml", "requirements.yaml", "galaxy.yml"],
        Yaml,
        "galaxy",
        &[
            Record("collections.*", "name", "version", ""),
            Keys("dependencies"),
        ],
    ),
    row(
        "terraform",
        &["*.tf", "*.tofu"],
        Text,
        "terraform",
        &[Lines(
            r#"(?m)source[ \t]*=[ \t]*"(?P<name>[\w-]+/[\w-]+(?:/[\w-]+)?)"[ \t]*\r?\n[ \t]*version[ \t]*=[ \t]*"(?P<version>[^"]+)""#,
        )],
    ),
    row(
        "terraform-lock",
        &[".terraform.lock.hcl"],
        Text,
        "terraform",
        &[Lines(
            r#"provider[ \t]+"registry\.terraform\.io/(?P<name>[\w-]+/[\w-]+)"[ \t]*\{\s*version[ \t]*=[ \t]*"(?P<version>[^"]+)""#,
        )],
    ),
    row(
        "gitlab-ci",
        &[".gitlab-ci.yml", ".gitlab-ci.yaml"],
        Yaml,
        "docker",
        &[
            Spec("image", DOCKER),
            Spec("image.name", DOCKER),
            Spec("services.*", DOCKER),
            Spec("services.*.name", DOCKER),
            Spec("*.image", DOCKER),
            Spec("*.image.name", DOCKER),
            Spec("*.services.*", DOCKER),
            Spec("*.services.*.name", DOCKER),
        ],
    ),
    row(
        "circleci",
        &[".circleci/config.yml", ".circleci/config.yaml"],
        Yaml,
        "docker",
        &[
            Spec("jobs.*.docker.*.image", DOCKER),
            Spec("executors.*.docker.*.image", DOCKER),
        ],
    ),
];
