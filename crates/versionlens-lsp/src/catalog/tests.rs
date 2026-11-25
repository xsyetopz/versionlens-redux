use regex_automata::meta::Regex;

use super::*;
use crate::extract::Rule::{Lines, Spec};
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
        for rule in file.rules {
            if let Spec(_, pattern) | Lines(pattern) = rule {
                assert!(Regex::new(pattern).is_ok(), "{}: {pattern}", file.id);
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

/// The files upstream `VersionLens` 1.3.0 (`gitlab.com/versionlens/vscode-versionlens`) and this
/// fork's multi-crate `master` each read, so dropping a row that one of them served fails here.
const PRIOR_FILES: &[&str] = &[
    // Upstream 1.3.0 provider defaults.
    "package.json",
    "deno.json",
    "pnpm-workspace.yaml",
    ".yarnrc.yml",
    "pubspec.yaml",
    "App.csproj",
    "App.fsproj",
    "Build.targets",
    "Directory.Build.props",
    "dub.json",
    "dub.selections.json",
    "Cargo.toml",
    "composer.json",
    "dockerfile",
    "api.dockerfile",
    "Dockerfile",
    "api.Dockerfile",
    "compose.yaml",
    "compose.yml",
    "dev.compose.yaml",
    "dev.compose.yml",
    "docker-compose.yaml",
    "docker-compose.yml",
    "pom.xml",
    "Pipfile",
    "pyproject.toml",
    "requirements.txt",
    "go.mod",
    "Gemfile",
    // The fork's multi-crate master, beyond upstream.
    ".bun-version",
    ".node-version",
    ".nvmrc",
    ".yarnrc.yaml",
    "CMakeLists.txt",
    "deps.cmake",
    "Cartfile",
    "Cartfile.private",
    "Chart.yaml",
    "DESCRIPTION",
    "kustomization.yaml",
    "kustomization.yml",
    "Kustomization",
    "MODULE.bazel",
    "Manifest.toml",
    "Manifest-v1.11.toml",
    "Package.swift",
    "Podfile",
    "Project.toml",
    "WORKSPACE",
    "WORKSPACE.bazel",
    "build.gradle",
    "build.gradle.kts",
    "build.sbt",
    "build.zig.zon",
    "app.cabal",
    "cabal.project",
    "conanfile.py",
    "conanfile.txt",
    "cpanfile",
    "deno.jsonc",
    "deps.edn",
    "dotnet-tools.json",
    "dub.sdl",
    "dune-project",
    "flake.nix",
    "app.gemspec",
    "gleam.toml",
    "go.work",
    "haxelib.json",
    "import_map.json",
    "jsr.json",
    "jsr.jsonc",
    "gradle/libs.versions.toml",
    "mix.exs",
    "app.nimble",
    "opam",
    "app.opam",
    "package.json5",
    "package.yaml",
    "package.yml",
    "packages.config",
    "paket.dependencies",
    "paket.references",
    "pnpm-workspace.yml",
    "project.clj",
    "project.json",
    "pubspec.yml",
    "pubspec_overrides.yaml",
    "rebar.config",
    "renv.lock",
    "requirements.yml",
    "requirements.yaml",
    "requirements-dev.txt",
    "foo-1.0-1.rockspec",
    "rust-toolchain",
    "rust-toolchain.toml",
    "settings.gradle",
    "settings.gradle.kts",
    "stack.yaml",
    "stack.yml",
    "main.tf",
    "main.tofu",
    "vcpkg.json",
    "xmake.lua",
    "subprojects/zlib.wrap",
    ".cargo/config.toml",
    ".github/workflows/ci.yml",
    "Packages/manifest.json",
];

#[test]
fn serves_every_file_upstream_and_master_served() {
    let unserved: Vec<&str> = PRIOR_FILES
        .iter()
        .copied()
        .filter(|path| file_type(path).is_none())
        .collect();
    assert!(unserved.is_empty(), "no file type for {unserved:?}");
}
