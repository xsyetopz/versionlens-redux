//! Patterns that more than one row uses.

use crate::extract::Rule::{self, Keys, Record, Spec, Via};

pub(super) const PEP508: &str = r"^\s*(?P<name>[A-Za-z0-9][A-Za-z0-9._-]*)\s*(?:\[[^\]]*\])?\s*\(?(?P<version>(?:===|==|~=|!=|<=|>=|<|>)\s*[^;,\s)]+(?:\s*,\s*(?:===|==|~=|!=|<=|>=|<|>)\s*[^;,\s)]+)*)?\)?\s*(?:;.*)?$";
pub(super) const DOCKER: &str = r"^(?P<name>[\w./-]+(?::\d+/[\w./-]+)?):(?P<version>[\w.-]+)$";
/// An `image:` or `container:` line naming a tagged image, as CI and dev-environment files write it.
pub(super) const IMAGE_LINE: &str = r#"(?m)^[ \t]*(?:-[ \t]*)?(?:image|container)[ \t]*:[ \t]*["']?(?P<name>[\w./-]+(?::\d+/[\w./-]+)?):(?P<version>[\w.-]+)["']?[ \t]*(?:#.*)?\r?$"#;
pub(super) const ACTION: &str = r"^(?P<name>[\w.-]+/[\w.-]+)(?:/[^@]*)?@(?P<version>v?\d[\w.-]*)$";
pub(super) const TOOL_VERSION: &str = r"\A\s*v?(?P<version>\d[\w.]*)\s*\z";
pub(super) const MAVEN_SPEC: &str = r"^(?P<name>[\w.-]+:[\w.-]+):(?P<version>[\w.+\[\](),-]+)$";
pub(super) const NPM_DEPS: &[Rule] = &[
    Keys("dependencies"),
    Keys("devDependencies"),
    Keys("peerDependencies"),
    Keys("optionalDependencies"),
    Keys("resolutions"),
    Keys("overrides"),
    Keys("pnpm.overrides"),
    Keys("catalog"),
    Keys("catalogs.*"),
    Keys("workspaces.catalog"),
    Keys("workspaces.catalogs.*"),
    Keys("volta"),
    Spec("packageManager", r"^(?P<name>[\w-]+)@(?P<version>[^+]+)"),
    // JSPM names its registry in each value: `npm:lodash@^4.17.0`, `github:owner/repo@1.0.0`.
    Spec(
        "jspm.*.*",
        r"^npm:(?P<name>(?:@[\w.-]+/)?[\w.-]+)@(?P<version>.+)$",
    ),
    Via(
        "github",
        &Spec(
            "jspm.*.*",
            r"^github:(?P<name>[\w.-]+/[\w.-]+)@(?P<version>.+)$",
        ),
    ),
];
pub(super) const PYPROJECT: &[Rule] = &[
    Spec("project.dependencies.*", PEP508),
    Spec("project.optional-dependencies.*.*", PEP508),
    Spec("dependency-groups.*.*", PEP508),
    Spec("build-system.requires.*", PEP508),
    Spec("tool.uv.dev-dependencies.*", PEP508),
    Spec("tool.uv.constraint-dependencies.*", PEP508),
    Spec("tool.uv.override-dependencies.*", PEP508),
    Spec("tool.pdm.dev-dependencies.*.*", PEP508),
    Keys("tool.poetry.dependencies"),
    Keys("tool.poetry.dev-dependencies"),
    Keys("tool.poetry.group.*.dependencies"),
];
pub(super) const NUGET: &[Rule] = &[
    Record(
        "Project.ItemGroup.PackageReference",
        "@Include|@Update",
        "@Version|Version",
        "",
    ),
    Record(
        "Project.ItemGroup.PackageVersion",
        "@Include|@Update",
        "@Version|Version",
        "",
    ),
    Record(
        "Project.ItemGroup.GlobalPackageReference",
        "@Include|@Update",
        "@Version|Version",
        "",
    ),
    Record("packages.package", "@id", "@version", ""),
];
pub(super) const POM: &[Rule] = &[
    Record("project.parent", "groupId:artifactId", "version", ""),
    Record(
        "project.dependencies.dependency",
        "groupId:artifactId",
        "version",
        "",
    ),
    Record(
        "project.dependencyManagement.dependencies.dependency",
        "groupId:artifactId",
        "version",
        "",
    ),
    Record(
        "project.build.plugins.plugin",
        "groupId:artifactId",
        "version",
        "",
    ),
    Record(
        "project.build.pluginManagement.plugins.plugin",
        "groupId:artifactId",
        "version",
        "",
    ),
];

/// A GitHub archive URL pinned to a tag, as Zig, Meson wraps, and Bazel `http_archive` write it.
pub(super) const GITHUB_ARCHIVE: &str = r"github\.com/(?P<name>[\w.-]+/[\w.-]+)/archive/(?:refs/tags/)?(?P<version>v?\d[\w.-]*?)\.(?:tar\.gz|tgz|zip|tar\.xz|tar\.bz2)";
/// A GitHub release download URL, whose path segment after `download/` is the tag.
pub(super) const GITHUB_RELEASE: &str =
    r"github\.com/(?P<name>[\w.-]+/[\w.-]+)/releases/download/(?P<version>v?\d[\w.-]*)/";
/// A GitHub repository URL with a `#tag` fragment, as in `build.zig.zon`.
pub(super) const GITHUB_FRAGMENT: &str =
    r"github\.com/(?P<name>[\w.-]+/[\w-]+(?:\.[\w-]+)*?)(?:\.git)?#(?P<version>v?\d[\w.-]*)";
