# Changelog

All notable changes to VersionLens Redux are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and releases follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://github.com/xsyetopz/vscode-versionlens/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/xsyetopz/vscode-versionlens/compare/v0.1.2...v0.2.0
[0.1.2]: https://github.com/xsyetopz/vscode-versionlens/compare/0.1.1...v0.1.2
[0.1.1]: https://github.com/xsyetopz/vscode-versionlens/compare/0.1.0...0.1.1
[0.1.0]: https://github.com/xsyetopz/vscode-versionlens/releases/tag/0.1.0
