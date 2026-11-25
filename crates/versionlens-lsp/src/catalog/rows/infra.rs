//! Containers, CI, orchestration, and infrastructure files.

use super::shared::{ACTION, DOCKER, IMAGE_LINE};
use crate::catalog::{FileType, row};
use crate::extract::Rule::{Keys, Lines, Record, Spec, Via};
use crate::flatten::Format::{Text, Yaml};

pub(super) static ROWS: &[FileType] = &[
    row(
        "dockerfile",
        &[
            "Dockerfile",
            "dockerfile",
            "*.Dockerfile",
            "*.dockerfile",
            "Dockerfile.*",
            "Containerfile",
            "Earthfile",
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
            Via("docker", &Lines(IMAGE_LINE)),
            Via(
                "docker",
                &Lines(
                    r#"(?m)uses:[ \t]*["']?docker://(?P<name>[\w./-]+(?::\d+/[\w./-]+)?):(?P<version>[\w.-]+)"#,
                ),
            ),
        ],
    ),
    // CI services and dev environments that name images under `image:` keys at any depth.
    row(
        "ci-images",
        &[
            "bitbucket-pipelines.yml",
            ".drone.yml",
            ".woodpecker.yml",
            ".woodpecker.yaml",
            ".woodpecker/*.yml",
            ".woodpecker/*.yaml",
            ".gitpod.yml",
            ".cirrus.yml",
            "azure-pipelines.yml",
            "azure-pipelines.yaml",
            "cloudbuild.yaml",
            "cloudbuild.yml",
        ],
        Text,
        "docker",
        &[Lines(IMAGE_LINE)],
    ),
    // Kubernetes manifests have no fixed name; these are the folders and suffixes that hold them.
    row(
        "kubernetes",
        &[
            "k8s/**/*.yaml",
            "k8s/**/*.yml",
            "kubernetes/**/*.yaml",
            "kubernetes/**/*.yml",
            "manifests/**/*.yaml",
            "manifests/**/*.yml",
            "deploy/**/*.yaml",
            "deploy/**/*.yml",
            "*.k8s.yaml",
            "*.k8s.yml",
            "skaffold.yaml",
            "tilt.yaml",
        ],
        Text,
        "docker",
        &[Lines(IMAGE_LINE)],
    ),
    // Helm chart values split an image into `repository` and `tag`, at the top or under a component.
    row(
        "helm-values",
        &[
            "values.yaml",
            "values.yml",
            "values-*.yaml",
            "values.*.yaml",
        ],
        Yaml,
        "docker",
        &[
            Lines(IMAGE_LINE),
            Record("image", "repository", "tag", ""),
            Record("*.image", "repository", "tag", ""),
        ],
    ),
    row(
        "puppetfile",
        &["Puppetfile"],
        Text,
        "puppet",
        &[Lines(
            r#"(?m)^[ \t]*mod[ \t]+["'](?P<group>[A-Za-z0-9]+)[-/](?P<name>[a-z][a-z0-9_]*)["'][ \t]*,[ \t]*["'](?P<version>\d[^"']*)["']"#,
        )],
    ),
    row(
        "chef",
        &["Berksfile", "metadata.rb"],
        Text,
        "chef",
        &[Lines(
            r#"(?m)^[ \t]*(?:cookbook|depends)[ \t]+["'](?P<name>[\w.-]+)["'][ \t]*,[ \t]*["'](?P<version>[^"']+)["']"#,
        )],
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
