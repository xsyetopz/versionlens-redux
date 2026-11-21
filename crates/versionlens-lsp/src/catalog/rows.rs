//! The `FILE_TYPES` table and the patterns its rows share.

use super::{FileType, row};
use crate::extract::Rule::{self, Keys, Lines, Record, Spec};
use crate::flatten::Format::{Json, Text, Toml, Xml, Yaml};

const PEP508: &str = r"^\s*(?P<name>[A-Za-z0-9][A-Za-z0-9._-]*)\s*(?:\[[^\]]*\])?\s*\(?(?P<version>(?:===|==|~=|!=|<=|>=|<|>)\s*[^;,\s)]+(?:\s*,\s*(?:===|==|~=|!=|<=|>=|<|>)\s*[^;,\s)]+)*)?\)?\s*(?:;.*)?$";
const DOCKER: &str = r"^(?P<name>[\w./-]+(?::\d+/[\w./-]+)?):(?P<version>[\w.-]+)$";
const ACTION: &str = r"^(?P<name>[\w.-]+/[\w.-]+)(?:/[^@]*)?@(?P<version>v?\d[\w.-]*)$";
const TOOL_VERSION: &str = r"\A\s*v?(?P<version>\d[\w.]*)\s*\z";
const MAVEN_SPEC: &str = r"^(?P<name>[\w.-]+:[\w.-]+):(?P<version>[\w.+\[\](),-]+)$";
const NPM_DEPS: &[Rule] = &[
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
    Spec("packageManager", r"^(?P<name>[\w-]+)@(?P<version>[^+]+)"),
];
const PYPROJECT: &[Rule] = &[
    Spec("project.dependencies.*", PEP508),
    Spec("project.optional-dependencies.*.*", PEP508),
    Spec("dependency-groups.*.*", PEP508),
    Spec("build-system.requires.*", PEP508),
    Keys("tool.poetry.dependencies"),
    Keys("tool.poetry.dev-dependencies"),
    Keys("tool.poetry.group.*.dependencies"),
];
const NUGET: &[Rule] = &[
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
const POM: &[Rule] = &[
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

pub(crate) static FILE_TYPES: &[FileType] = &[
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
        &["pnpm-workspace.yaml", "pnpm-workspace.yml", "package.yaml"],
        Yaml,
        "npm",
        NPM_DEPS,
    ),
    row(
        "deno",
        &["deno.json", "deno.jsonc", "import_map.json"],
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
        "go",
        &["go.mod", "go.work"],
        Text,
        "go",
        &[Lines(
            r"(?m)^(?:require[ \t]+|[ \t]+)(?P<name>[\w.~-]+\.[\w.~/-]+)[ \t]+(?P<version>v\d[^\s]*)",
        )],
    ),
    row(
        "gemfile",
        &["Gemfile", "*.gemspec"],
        Text,
        "rubygems",
        &[Lines(
            r#"(?m)^[ \t]*(?:gem|\w+\.add_(?:runtime_|development_)?dependency)[ \t(]+["'](?P<name>[^"']+)["'][ \t]*,[ \t]*["'](?P<version>[^"']+)["']"#,
        )],
    ),
    row(
        "podfile",
        &["Podfile"],
        Text,
        "cocoapods",
        &[Lines(
            r#"(?m)^[ \t]*pod[ \t]+["'](?P<name>[^"']+)["'][ \t]*,[ \t]*["'](?P<version>[^"']+)["']"#,
        )],
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
        "paket",
        &["paket.dependencies"],
        Text,
        "nuget",
        &[Lines(
            r"(?m)^[ \t]*nuget[ \t]+(?P<name>[\w.-]+)[ \t]+(?P<version>(?:[~><=]+[ \t]*)?\d[\w.-]*)",
        )],
    ),
    row(
        "nuget",
        &[
            "*.csproj",
            "*.fsproj",
            "*.vbproj",
            "*.props",
            "*.targets",
            "packages.config",
        ],
        Xml,
        "nuget",
        NUGET,
    ),
    row(
        "dotnet-tools",
        &["dotnet-tools.json"],
        Json,
        "nuget",
        &[Keys("tools")],
    ),
    row("maven", &["pom.xml"], Xml, "maven", POM),
    row(
        "gradle",
        &[
            "build.gradle",
            "build.gradle.kts",
            "settings.gradle",
            "settings.gradle.kts",
        ],
        Text,
        "maven",
        &[Lines(
            r#"["'](?P<name>[\w.-]+:[\w.-]+):(?P<version>[\w.+\[\](),-]+)["']"#,
        )],
    ),
    row(
        "gradle-catalog",
        &["libs.versions.toml", "*.versions.toml"],
        Toml,
        "maven",
        &[
            Spec("libraries.*", MAVEN_SPEC),
            Record("libraries.*", "module", "version", ""),
            Record("libraries.*", "group:name", "version", ""),
        ],
    ),
    row(
        "sbt",
        &["build.sbt"],
        Text,
        "maven",
        &[Lines(
            r#""(?P<group>[\w.-]+)"[ \t]*%[ \t]*"(?P<name>[\w.-]+)"[ \t]*%[ \t]*"(?P<version>[\w.+-]+)""#,
        )],
    ),
    row(
        "deps-edn",
        &["deps.edn"],
        Text,
        "maven",
        &[Lines(
            r#"(?P<name>[\w.-]+/[\w.-]+)\s+\{\s*:mvn/version\s+"(?P<version>[^"]+)""#,
        )],
    ),
    row(
        "leiningen",
        &["project.clj"],
        Text,
        "clojars",
        &[Lines(
            r#"\[(?P<name>[\w.-]+(?:/[\w.-]+)?)\s+"(?P<version>[^"]+)""#,
        )],
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
        &["Project.toml", "JuliaProject.toml"],
        Toml,
        "julia",
        &[Keys("compat")],
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
        &["kustomization.yaml", "kustomization.yml"],
        Yaml,
        "docker",
        &[Record("images.*", "name", "newTag", "")],
    ),
    row(
        "github-actions",
        &[
            ".github/workflows/*.yml",
            ".github/workflows/*.yaml",
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
        &[Record("collections.*", "name", "version", "")],
    ),
    row(
        "terraform",
        &["*.tf", "*.tofu"],
        Text,
        "terraform",
        &[Lines(
            r#"(?m)source[ \t]*=[ \t]*"(?P<name>[\w-]+/[\w-]+)"[ \t]*\r?\n[ \t]*version[ \t]*=[ \t]*"(?P<version>[^"]+)""#,
        )],
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
        "carthage",
        &["Cartfile"],
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
        "github",
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
        &["*.cabal", "cabal.project"],
        Text,
        "hackage",
        &[Lines(
            r"(?m)(?:build-depends:|,|^)[ \t]*(?P<name>[A-Za-z][\w-]*)[ \t]+(?P<version>(?:\^>=|>=|==)[ \t]*\d[\d.]*)",
        )],
    ),
];
