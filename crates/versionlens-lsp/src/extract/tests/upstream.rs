//! The dependency properties upstream `VersionLens` 1.3.0 reads, so dropping one fails here.

use super::deps;

/// Each dependency property upstream `VersionLens` 1.3.0 reads by default, as one manifest per
/// provider whose dependency names say which property holds them.
const UPSTREAM_PROPERTIES: &[(&str, &str, &[&str])] = &[
    (
        "package.json",
        r#"{"packageManager": "pnpm@9.0.0", "dependencies": {"dep": "1.0.0"}, "devDependencies": {"dev": "1.0.0"}, "peerDependencies": {"peer": "1.0.0"}, "optionalDependencies": {"optional": "1.0.0"}, "overrides": {"override": "1.0.0", "parent": {"nested": "1.0.0"}}, "jspm": {"dependencies": {"jspm-dep": "npm:jd@1.0.0"}, "devDependencies": {"jspm-dev": "npm:jv@1.0.0"}, "peerDependencies": {"jspm-peer": "npm:jp@1.0.0"}, "optionalDependencies": {"jspm-optional": "npm:jo@1.0.0"}}, "pnpm": {"overrides": {"pnpm-override": "1.0.0", "pp": {"pnpm-nested": "1.0.0"}}}, "catalog": {"cat": "1.0.0"}, "catalogs": {"x": {"cats": "1.0.0"}}, "workspaces": {"catalog": {"ws-cat": "1.0.0"}, "catalogs": {"y": {"ws-cats": "1.0.0"}}}}"#,
        &[
            "pnpm",
            "dep",
            "dev",
            "peer",
            "optional",
            "override",
            "nested",
            "jd",
            "jv",
            "jp",
            "jo",
            "pnpm-override",
            "pnpm-nested",
            "cat",
            "cats",
            "ws-cat",
            "ws-cats",
        ],
    ),
    (
        "deno.json",
        r#"{"imports": {"std/": "jsr:@std/path@^1.0.0", "chalk": "npm:chalk@5.0.0"}}"#,
        &["@std/path", "chalk"],
    ),
    (
        "pnpm-workspace.yaml",
        "catalog:\n  cat: 1.0.0\ncatalogs:\n  x:\n    cats: 1.0.0\noverrides:\n  override: 1.0.0\npackageExtensions:\n  host:\n    dependencies:\n      ext-dep: 1.0.0\n    devDependencies:\n      ext-dev: 1.0.0\n    peerDependencies:\n      ext-peer: 1.0.0\n    optionalDependencies:\n      ext-optional: 1.0.0\n",
        &[
            "cat",
            "cats",
            "override",
            "ext-dep",
            "ext-dev",
            "ext-peer",
            "ext-optional",
        ],
    ),
    (
        ".yarnrc.yml",
        "catalog:\n  cat: 1.0.0\ncatalogs:\n  x:\n    cats: 1.0.0\n",
        &["cat", "cats"],
    ),
    (
        "pubspec.yaml",
        "dependencies:\n  dep: ^1.0.0\ndev_dependencies:\n  dev: ^1.0.0\ndependency_overrides:\n  override: ^1.0.0\n",
        &["dep", "dev", "override"],
    ),
    (
        "App.csproj",
        r#"<Project Sdk="MSBuild.Sdk.Extras/3.0.44"><ItemGroup><GlobalPackageReference Include="Global" Version="1.0.0" /><PackageReference Include="Ref" Version="1.0.0" /><PackageVersion Include="Central" Version="1.0.0" /><DotNetCliToolReference Include="Tool" Version="1.0.0" /></ItemGroup></Project>"#,
        &["MSBuild.Sdk.Extras", "Global", "Ref", "Central", "Tool"],
    ),
    (
        "dub.json",
        r#"{"dependencies": {"vibe-d": "~>0.9.0"}}"#,
        &["vibe-d"],
    ),
    (
        "dub.selections.json",
        r#"{"fileVersion": 1, "versions": {"vibe-core": "2.0.0"}}"#,
        &["vibe-core"],
    ),
    (
        "Cargo.toml",
        "[dependencies]\ndep = \"1\"\n[dependencies.table]\nversion = \"1\"\n[dev-dependencies]\ndev = \"1\"\n[dev-dependencies.dev-table]\nversion = \"1\"\n[build-dependencies]\nbuild = \"1\"\n[build-dependencies.build-table]\nversion = \"1\"\n[workspace.dependencies]\nws = \"1\"\n[workspace.dependencies.ws-table]\nversion = \"1\"\n",
        &[
            "dep",
            "table",
            "dev",
            "dev-table",
            "build",
            "build-table",
            "ws",
            "ws-table",
        ],
    ),
    (
        "composer.json",
        r#"{"require": {"vendor/req": "^1.0"}, "require-dev": {"vendor/dev": "^1.0"}}"#,
        &["vendor/req", "vendor/dev"],
    ),
    ("Dockerfile", "FROM node:20 AS build\n", &["node"]),
    (
        "compose.yaml",
        "services:\n  db:\n    image: postgres:16\n",
        &["postgres"],
    ),
    (
        "pom.xml",
        "<project><parent><groupId>org.springframework.boot</groupId><artifactId>spring-boot-starter-parent</artifactId><version>3.0.0</version></parent><dependencies><dependency><groupId>g</groupId><artifactId>a</artifactId><version>1.0</version></dependency></dependencies></project>",
        &["org.springframework.boot:spring-boot-starter-parent", "g:a"],
    ),
    (
        "Pipfile",
        "[packages]\nrequests = \"==2.0.0\"\n[dev-packages]\npytest = \"==7.0.0\"\n",
        &["requests", "pytest"],
    ),
    (
        "pyproject.toml",
        "[project]\ndependencies = [\"dep>=1.0\"]\n[project.optional-dependencies]\nextra = [\"opt>=1.0\"]\n[dependency-groups]\ndev = [\"grouped>=1.0\"]\n[tool.poetry.dependencies]\npoetry-dep = \"^1.0\"\n[tool.poetry.group.test.dependencies]\npoetry-group = \"^1.0\"\n",
        &["dep", "opt", "grouped", "poetry-dep", "poetry-group"],
    ),
    ("requirements.txt", "requests==2.0.0\n", &["requests"]),
    (
        "go.mod",
        "module m\n\nrequire golang.org/x/text v0.14.0\n",
        &["golang.org/x/text"],
    ),
    ("Gemfile", "gem 'rails', '~> 7.0'\n", &["rails"]),
];

#[test]
fn reads_every_upstream_dependency_property() {
    let missing: Vec<String> = UPSTREAM_PROPERTIES
        .iter()
        .flat_map(|&(path, text, names)| {
            let found: Vec<String> = deps(path, text).into_iter().map(|(n, _)| n).collect();
            names
                .iter()
                .filter(move |n| !found.iter().any(|f| f == *n))
                .map(move |n| format!("{path} {n}"))
        })
        .collect();
    assert!(missing.is_empty(), "not read: {missing:?}");
}
