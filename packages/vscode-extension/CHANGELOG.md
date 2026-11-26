# Changelog

## Unreleased

- Replaced the per-provider settings with one flat `versionlens` section; see the README's Settings table. Settings apply without a restart, and every command acts on the active manifest. The full list of changes is in the [repository changelog](../../CHANGELOG.md).

## 0.5.0

- Grew these defaults for new manifest support. A setting you have set yourself does not gain the new entries, so add them by hand: `versionlens.cargo.files` (`**/.cargo/config.toml`, `**/.cargo/config`), `versionlens.dotnet.files` (`**/dotnet-tools.json`), `versionlens.swift.files` (`**/Cartfile`, `**/Cartfile.private`) and `versionlens.cargo.dependencyProperties` (`patch.*`, `patch.*.*`).

## 0.4.4

- Made cold dependency checks substantially faster across providers with concurrent reusable HTTP connections, shared bodies, coalesced work, a compressed CRAN catalog, shared LuaRocks catalog requests, and smaller official npm responses while preserving complete release-history semantics.

## 0.1.0

### VersionLens Redux

- Rebranded the VS Code extension as `versionlens-redux` under publisher `xsyetopz`.
- Added conflict detection for the original `pflannery.vscode-versionlens` extension.
- Kept attribution to the original VersionLens creators while tracking this fork under the Redux identity.
- Expanded manifest support and Rust-backed analysis for additional dependency ecosystems, including broader C/C++ and JVM build file coverage.
