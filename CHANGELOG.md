# Changelog

All notable changes to VersionLens Redux are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and releases follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0] - 2026-09-01

### Added

- Added workspace-aware local package resolution for npm, Bun, pnpm, Yarn, and Cargo projects while keeping ambiguous or external packages on their existing registry paths.
- Added validated multi-document workspace edit plans that preserve open-document text and reject stale, duplicate, or overlapping edits before applying coordinated package-version changes.
- Added version lenses for additional versionable metadata, including package runtime engines, Cargo Rust toolchains, Kotlin Gradle plugins, and surfaced workspace or catalog references.
- Added actionable downgrade lenses for every known older stable release, with a configurable downward indicator and existing prerelease visibility rules.
- Added a production-ready Neovim plugin with automatic LSP startup, code-lens refresh, health checks, Plenary tests, platform-specific release archives, and stable/nightly CI coverage.

### Changed

- Made the shared Rust model the source of truth for dependency, project-version, runtime-constraint, workspace-reference, and ecosystem-handle classification across native and editor boundaries.
- Made open workspace documents authoritative over stale on-disk manifests during local package discovery and coordinated edits.
- Kept all Rust crates and VS Code, Zed, Neovim, and JetBrains packages aligned on version 0.4.0.
- Made JetBrains release archives Marketplace-selectable by OS and architecture, added plugin icons and verifier metadata, and hardened native language-server extraction.
- Resolved GitHub Actions tags through their canonical semantic version while preserving repository-specific prefixes such as `v`, `action-v`, and path-qualified tag namespaces.

### Fixed

- Removed redundant latest-version actions when a dependency or accepted range already resolves to the true latest release.
- Treated exact GitHub Action tags with `v` or `V` prefixes as current when they match the latest release, while preserving the prefix in displayed and applied version changes.
- Kept selected lower versions actionable through the existing safe edit path instead of rejecting valid downgrades.
- Tracked full and abbreviated GitHub Actions commit pins when an adjacent version comment identifies the exact repository tag, updating the commit SHA and comment together without replacing the immutable pin with a mutable tag.
- Rejected mismatched, ambiguous, malformed, bare-SHA, branch, and local Action references unless their identity and edit semantics are proven.
- Preserved established safe behavior for indirect references, including npm and Ruby Git refs, Maven properties, Gradle catalogs, workspace references, and immutable Docker digest pins.
- Completed the workspace and edit-plan boundary tests and architecture ownership required for the 0.4.0 release gates.

## [0.3.0] - 2026-08-28

### Added

- Added GitHub Actions workflow and reusable-workflow version detection through repository tag lookups.
- Added repository-owned capability and architecture audits for supported parsers, providers, native boundaries, and editor packages.

### Changed

- Split parser, provider, edit, suggestion, core-session, native, and editor responsibilities into explicit shared contracts and independently checked boundaries.
- Centralized release version management, packaging helpers, toolchain setup, fixtures, and editor package validation.

## [0.2.0] - 2026-07-19

### Added

- Added actionable latest, major, minor, patch, range-bump, prerelease, build, and vulnerability-aware upgrade choices across supported manifests.
- Added first-class parsing and provider coverage for additional native, JVM, scripting, infrastructure, and package-manager ecosystems.
- Added an explicit shared model crate and typed Rust, N-API, and VS Code host boundaries.

### Changed

- Reorganized the Rust workspace, provider pipeline, N-API bindings, extension adapter, and test ownership around enforceable module boundaries.
- Updated the Rust and Bun dependency sets and migrated the repository to the current Biome configuration without lint suppressions.
- Hardened release packaging, editor-package checks, dependency caching, registry resolution, and authentication handling.

### Fixed

- Restored upward-arrow upgrade lenses for outdated fixed versions, ranges, invalid requirements, and Python project dependencies.
- Preserved standard status and upgrade glyphs when configured indicators are missing, empty, or whitespace-only.
- Made every displayed upgrade choice apply the selected version while preserving valid Python, Ruby, SemVer, and manifest syntax.
- Preserved quoted, escaped, and nested manifest structures when sorting dependencies.
- Kept Rust and TypeScript authorization outputs synchronized and required across the native boundary.

## [0.1.2] - 2026-07-12

### Added

- Added a single command to build the VS Code, Zed, and JetBrains packages.
- Bundled and verified each editor package's required native runtime.
- Added a gated GitHub release workflow that tags successful master builds and attaches all three editor packages.
- Added target-specific VSIX packages for every native VS Code desktop target.

### Fixed

- Report incompatible VS Code native runtimes explicitly instead of failing activation without a VersionLens message.
- Package the correct native runtime for Windows, Linux, Alpine Linux, and macOS across x64, ARM64, and Linux ARMv7 targets.

## [0.1.1] - 2026-07-12

### Added

- Added a strict SemVer repository version-bump command covering Rust, Bun, VS Code, Zed, and JetBrains manifests and lockfiles.

### Fixed

- Parsed parenthesized PEP 508 requirements without including version syntax in Python package names.
- Escaped unsafe registry URL bytes and replaced every configured URL template placeholder.
- Made the VS Code version-lens toggle resolve the active document directly.
- Updated LSP response construction for lsp-server 0.9.

## [0.1.0] - 2026-07-11

### Added

- Introduced VersionLens Redux as the versionlens-redux VS Code extension under the xsyetopz publisher.
- Added conflict detection for the original pflannery.vscode-versionlens extension.
- Added Rust-backed dependency analysis across the supported manifest ecosystems, including C/C++ and JVM build files.
- Preserved attribution to the original VersionLens authors.

[Unreleased]: https://github.com/xsyetopz/versionlens-redux/compare/v0.4.0...HEAD
[0.4.0]: https://github.com/xsyetopz/versionlens-redux/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/xsyetopz/versionlens-redux/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/xsyetopz/versionlens-redux/compare/v0.1.2...v0.2.0
[0.1.2]: https://github.com/xsyetopz/versionlens-redux/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/xsyetopz/versionlens-redux/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/xsyetopz/versionlens-redux/releases/tag/v0.1.0
