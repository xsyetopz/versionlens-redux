//! .NET and JVM manifests.

use super::shared::{MAVEN_SPEC, NUGET, POM};
use crate::catalog::{FileType, row};
use crate::extract::Rule::{Keys, Lines, Record, Spec};
use crate::flatten::Format::{Json, Text, Toml, Xml};

pub(super) static ROWS: &[FileType] = &[
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
        &["*.sbt"],
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
        "clojars",
        &[Lines(
            r#"(?P<name>[\w.-]+/[\w.-]+)\s+\{\s*:mvn/version\s+"(?P<version>[^"]+)""#,
        )],
    ),
    row(
        "ivy",
        &["ivy.xml"],
        Xml,
        "maven",
        &[Record(
            "ivy-module.dependencies.dependency",
            "@org:@name",
            "@rev",
            "",
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
        "dotnet-project-json",
        &["project.json"],
        Json,
        "nuget",
        &[Keys("dependencies"), Keys("frameworks.*.dependencies")],
    ),
    row(
        "global-json",
        &["global.json"],
        Json,
        "nuget",
        &[Keys("msbuild-sdks")],
    ),
    row(
        "maven-extensions",
        &[".mvn/extensions.xml"],
        Xml,
        "maven",
        &[Record(
            "extensions.extension",
            "groupId:artifactId",
            "version",
            "",
        )],
    ),
    row(
        "kotlin-script",
        &["*.main.kts"],
        Text,
        "maven",
        &[Lines(
            r#"@file:DependsOn\("(?P<name>[\w.-]+:[\w.-]+):(?P<version>[\w.+\[\](),-]+)"\)"#,
        )],
    ),
    row(
        "gradle-wrapper",
        &["gradle-wrapper.properties"],
        Text,
        "gradle-dist",
        &[Lines(
            r"distributionUrl=.*?/(?P<name>gradle)-(?P<version>\d[\w.-]*?)-(?:bin|all)\.zip",
        )],
    ),
];
