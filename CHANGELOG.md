# Changelog

All notable changes to VersionLens Redux are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and releases follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://github.com/xsyetopz/vscode-versionlens/compare/v0.1.2...HEAD
[0.1.2]: https://github.com/xsyetopz/vscode-versionlens/compare/0.1.1...v0.1.2
[0.1.1]: https://github.com/xsyetopz/vscode-versionlens/compare/0.1.0...0.1.1
[0.1.0]: https://github.com/xsyetopz/vscode-versionlens/releases/tag/0.1.0
