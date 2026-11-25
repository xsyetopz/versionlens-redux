use regex_automata::meta::Regex;

use super::*;
use crate::extract::Rule::{Affix, Lines, Spec, Via, Within};
use crate::extract::{DIRECTORY, PROJECT, SOURCE, extract};
use crate::registry;

#[test]
fn matches_trailing_segments_case_insensitively() {
    let id = |path: &str| file_type(path).map(|f| f.id);
    assert_eq!(id("/a/requirements.txt"), Some("requirements"));
    assert_eq!(id("/a/requirements/dev.txt"), Some("requirements"));
    assert_eq!(id("C:\\a\\Requirements-Dev.TXT"), Some("requirements"));
    assert_eq!(id("/a/libraries.hxml"), Some("hxml"));
    assert_eq!(id("/a/.github/workflows/ci.yml"), Some("github-actions"));
    assert_eq!(id("/a/workflows/ci.yml"), None);
    assert_eq!(id("/a/Packages/manifest.json"), Some("unity"));
    assert_eq!(id("/a/notes.txt"), None);
}

#[test]
fn every_row_names_a_known_registry_and_valid_regexes() {
    for file in FILE_TYPES.iter() {
        assert!(
            registry::get(file.registry).is_some(),
            "{} names {}",
            file.id,
            file.registry
        );
        for mut rule in file.rules {
            if let Via(id, inner) = rule {
                assert!(
                    registry::get(id).is_some() || [DIRECTORY, PROJECT, SOURCE].contains(id),
                    "{} sends a rule to {id}",
                    file.id
                );
                rule = inner;
            }
            if let Affix(_, _, inner) = rule {
                rule = inner;
            }
            if let Within(block, inner) = rule {
                assert!(Regex::new(block).is_ok(), "{}: {block}", file.id);
                rule = inner;
            }
            if let Spec(_, pattern) | Lines(pattern) = rule
                && let Err(error) = Regex::new(pattern)
            {
                panic!("{}: {pattern}: {error}", file.id);
            }
        }
    }
    assert!(globs_json().contains("*.hxml"));
}

#[test]
fn matches_restored_and_added_globs() {
    let id = |path: &str| file_type(path).map(|f| f.id);
    assert_eq!(id("a/dockerfile"), Some("dockerfile"));
    assert_eq!(id("a/api.dockerfile"), Some("dockerfile"));
    assert_eq!(id("compose.dev.yaml"), Some("compose"));
    assert_eq!(id(".github/workflows/a/b.yml"), Some("github-actions"));
    assert_eq!(id(".github/actions/x/action.yml"), Some("github-actions"));
    assert_eq!(
        id("a/.github/actions/x/y/action.yaml"),
        Some("github-actions")
    );
    assert_eq!(id("rust-toolchain"), Some("rust"));
    assert_eq!(id("a/.cargo/config.toml"), Some("cargo-config"));
    assert_eq!(id("flake.nix"), Some("flake"));
    assert_eq!(id("x/foo-1.0-1.rockspec"), Some("rockspec"));
    assert_eq!(id("xmake.lua"), Some("xmake"));
    assert_eq!(id("subprojects/a.wrap"), Some("meson-wrap"));
    assert_eq!(id("a.wrap"), None);
    assert_eq!(id(".mise/config.toml"), Some("mise"));
    assert_eq!(id(".tool-versions"), Some("tool-versions"));
    assert_eq!(id(".python-version"), Some("python"));
}

/// Each file upstream `VersionLens` 1.3.0 (`gitlab.com/versionlens/vscode-versionlens`) and this
/// fork's multi-crate `master` read, with the row that reads it now, so dropping or moving a row
/// one of them served fails here.
const PRIOR_FILES: &[(&str, &str)] = &[
    // Upstream 1.3.0 provider defaults.
    ("package.json", "npm"),
    ("deno.json", "deno"),
    ("pnpm-workspace.yaml", "pnpm"),
    (".yarnrc.yml", "pnpm"),
    ("pubspec.yaml", "pubspec"),
    ("App.csproj", "nuget"),
    ("App.fsproj", "nuget"),
    ("Build.targets", "nuget"),
    ("Directory.Build.props", "nuget"),
    ("dub.json", "dub"),
    ("dub.selections.json", "dub"),
    ("Cargo.toml", "cargo"),
    ("composer.json", "composer"),
    ("dockerfile", "dockerfile"),
    ("api.dockerfile", "dockerfile"),
    ("Dockerfile", "dockerfile"),
    ("api.Dockerfile", "dockerfile"),
    ("compose.yaml", "compose"),
    ("compose.yml", "compose"),
    ("dev.compose.yaml", "compose"),
    ("dev.compose.yml", "compose"),
    ("docker-compose.yaml", "compose"),
    ("docker-compose.yml", "compose"),
    ("pom.xml", "maven"),
    ("Pipfile", "pipfile"),
    ("pyproject.toml", "pyproject"),
    ("requirements.txt", "requirements"),
    ("go.mod", "go"),
    ("Gemfile", "gemfile"),
    // The fork's multi-crate master, from its classify.rs tests.
    ("PACKAGE.JSON", "npm"),
    ("package.json5", "npm"),
    ("package.yaml", "pnpm"),
    ("package.yml", "pnpm"),
    ("pnpm-workspace.yml", "pnpm"),
    ("PNPM-WORKSPACE.YAML", "pnpm"),
    (".yarnrc.yaml", "pnpm"),
    ("deno.jsonc", "deno"),
    ("import_map.json", "deno"),
    ("jsr.json", "deno"),
    ("jsr.jsonc", "deno"),
    ("project.json", "dotnet-project-json"),
    ("packages.config", "nuget"),
    ("App.vbproj", "nuget"),
    ("app.csproj", "nuget"),
    ("app.fsproj", "nuget"),
    ("app.vbproj", "nuget"),
    ("Directory.Packages.props", "nuget"),
    ("Directory.Build.targets", "nuget"),
    ("paket.dependencies", "paket"),
    ("paket.references", "paket-references"),
    (".config/dotnet-tools.json", "dotnet-tools"),
    ("dub.sdl", "dub-sdl"),
    ("vcpkg.json", "vcpkg"),
    ("CMakeLists.txt", "cmake"),
    ("toolchain.cmake", "cmake"),
    ("xmake.lua", "xmake"),
    ("Package.swift", "swift"),
    ("build.zig.zon", "zig"),
    ("demo.nimble", "nimble"),
    ("cpanfile", "cpanfile"),
    ("luasocket-3.1.0-1.rockspec", "rockspec"),
    ("requirements-dev.txt", "requirements"),
    ("constraints.txt", "requirements"),
    ("gradle/libs.versions.toml", "gradle-catalog"),
    ("build.gradle", "gradle"),
    ("build.gradle.kts", "gradle"),
    ("settings.gradle", "gradle"),
    ("settings.gradle.kts", "gradle"),
    ("build.sbt", "sbt"),
    ("deps.edn", "deps-edn"),
    ("project.clj", "leiningen"),
    (".nvmrc", "node"),
    (".node-version", "node"),
    (".bun-version", "bun"),
    ("rust-toolchain", "rust"),
    ("rust-toolchain.toml", "rust"),
    ("build.Dockerfile", "dockerfile"),
    ("build.dockerfile", "dockerfile"),
    ("DOCKERFILE", "dockerfile"),
    ("build.DOCKERFILE", "dockerfile"),
    ("DOCKER-COMPOSE.OVERRIDE.YML", "compose"),
    ("SERVICE.COMPOSE.YML", "compose"),
    ("service.compose.yaml", "compose"),
    (".YARNRC.YML", "pnpm"),
    ("docker-compose.override.yml", "compose"),
    ("docker-compose.prod.yaml", "compose"),
    ("compose.override.yaml", "compose"),
    ("service.compose.yml", "compose"),
    ("COMPOSE.YAML", "compose"),
    ("Chart.yaml", "helm"),
    ("requirements.yml", "ansible"),
    ("requirements.yaml", "ansible"),
    ("MODULE.bazel", "bazel"),
    ("WORKSPACE", "bazel-workspace"),
    ("WORKSPACE.bazel", "bazel-workspace"),
    ("subprojects/zlib.wrap", "meson-wrap"),
    ("flake.nix", "flake"),
    ("kustomization.yaml", "kustomize"),
    ("kustomization.yml", "kustomize"),
    ("Kustomization", "kustomize"),
    ("Packages/manifest.json", "unity"),
    ("example.gemspec", "gemfile"),
    ("go.work", "go"),
    ("mix.exs", "mix"),
    ("rebar.config", "rebar"),
    ("gleam.toml", "gleam"),
    ("opam", "opam"),
    ("lwt.opam", "opam"),
    ("dune-project", "dune"),
    ("demo.cabal", "cabal"),
    ("cabal.project", "cabal"),
    ("stack.yaml", "cabal"),
    ("stack.yml", "cabal"),
    ("Project.toml", "julia"),
    ("Manifest.toml", "julia-manifest"),
    ("Manifest-v1.11.toml", "julia-manifest"),
    ("Manifest-v1.10.toml", "julia-manifest"),
    ("pubspec.yml", "pubspec"),
    ("pubspec_overrides.yaml", "pubspec"),
    ("DESCRIPTION", "r"),
    ("renv.lock", "renv"),
    (".github/workflows/ci.yml", "github-actions"),
    (".github/actions/release/action.yaml", "github-actions"),
    ("main.tf", "terraform"),
    ("providers.tofu", "terraform"),
    ("conanfile.txt", "conan"),
    ("conanfile.py", "conan"),
    ("Cartfile", "carthage"),
    ("Cartfile.private", "carthage"),
    ("haxelib.json", "haxelib"),
    ("Podfile", "podfile"),
    (".cargo/config.toml", "cargo-config"),
];

/// The paths both baselines left unread: near-miss names, workflow folders' other files, and
/// the generated .props and .targets that .NET restore writes under `obj/` and `bin/`.
const PRIOR_UNREAD: &[&str] = &[
    "mycomposer.json",
    "notpackage.json",
    "appgo.mod",
    "testpom.xml",
    "otherpubspec.yaml",
    "ci.yml",
    ".github/workflows/README.md",
    "obj/project.assets.props",
    "bin/Debug/net8.0/app.targets",
    "OBJ/Debug/net8.0/generated.props",
    "BIN/Debug/net8.0/generated.targets",
];

#[test]
fn reads_every_file_upstream_and_master_read_with_the_same_row() {
    let wrong: Vec<String> = PRIOR_FILES
        .iter()
        .filter_map(|(path, want)| {
            let got = file_type(&format!("/work/{path}")).map(|f| f.id);
            (got != Some(*want)).then(|| format!("{path}: {got:?}, not {want}"))
        })
        .collect();
    assert!(wrong.is_empty(), "{wrong:#?}");
    let read: Vec<&str> = PRIOR_UNREAD
        .iter()
        .copied()
        .filter(|path| file_type_of(&format!("/work/{path}"), "{}").is_some())
        .collect();
    assert!(read.is_empty(), "read {read:?}");
}

#[test]
fn reads_json_shaped_like_package_json_as_npm() {
    for text in [
        r#"{ "devDependencies": { "typescript": "^6.0.3" } }"#,
        r#"{ "jspm": { "dependencies": { "systemjs": "^6.0.0" } } }"#,
        r#"{ "workspaces": { "catalog": { "react": "^19.0.0" } } }"#,
    ] {
        let file = file_type_of("/work/web-module.json", text).map(|f| (f.id, f.registry));
        assert_eq!(file, Some(("npm", "npm")), "{text}");
    }
    assert!(file_type("/work/web-module.json").is_none());
    assert!(file_type_of("/work/tsconfig.json", r#"{ "compilerOptions": {} }"#).is_none());
    // A row that matches by name wins over the content rows.
    let deno = file_type_of("/work/deno.json", r#"{ "dependencies": {} }"#);
    assert_eq!(deno.map(|f| f.id), Some("deno"));
}

/// The old master's checking-coverage cases where the server now names the dependency or spans
/// its requirement differently, each with the pair it reads now and why.
const RENAMED: &[(&str, &str, &str)] = &[
    // Clojure tools write `group/artifact`; the clojars registry maps it to Maven's form.
    ("ClojureDepsEdn", "example.test/foo", "1.0.0"),
    ("LeiningenProjectClj", "example.test/foo", "1.0.0"),
    // The operator sits outside the string opam edits, so the span holds the version alone; `=`
    // still pins and `>=` still bounds, as forms.rs's opam test shows.
    ("Opam", "foo", "1.0.0"),
    // Git-hosted dependencies are named by the repository whose tags answer for them.
    ("Cmake", "example/foo", "1.0.0"),
    ("MesonWrap", "example/foo", "1.0.0"),
    ("BazelWorkspace", "example/foo", "1.0.0"),
    ("SwiftPackage", "example/foo", "1.0.0"),
    ("ZigBuildZon", "example/foo", "1.0.0"),
    // A plugin goes by its id; the gradle-plugins registry finds its marker artifact.
    ("GradleSettings", "example.foo", "1.0.0"),
    // Tags keep their `v`, and a `v` written in .nvmrc stays outside the span.
    ("GitHubActions", "example/foo", "v1.0.0"),
    ("Nvmrc", "node", "1.0.0"),
];

#[test]
fn reads_what_master_checked_in_each_manifest_kind() {
    let cases: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("tests/manifest-cases.json")).expect("valid cases");
    let mut wrong = Vec::new();
    for case in &cases {
        let field = |key: &str| case[key].as_str().unwrap_or_default();
        let (kind, path, text) = (field("kind"), field("path"), field("text"));
        let want = RENAMED.iter().find(|(k, ..)| *k == kind).map_or_else(
            || (field("package"), field("requirement")),
            |&(_, name, req)| (name, req),
        );
        let found = file_type_of(path, text).map_or_else(Vec::new, |file| extract(file, text));
        let read = found
            .iter()
            .any(|d| (d.name.as_str(), d.req.as_str()) == want);
        if !read {
            wrong.push(format!("{kind} {path}: want {want:?}, read {read}"));
        }
    }
    assert_eq!(cases.len(), 74);
    assert!(wrong.is_empty(), "{wrong:#?}");
}
