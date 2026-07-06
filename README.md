# VersionLens Redux

VersionLens Redux shows dependency version information in editor manifests using a Rust core shared by VS Code, Zed, and JetBrains adapters.

[![VS Code CI/CD](https://github.com/xsyetopz/versionlens-redux/actions/workflows/vscode.yml/badge.svg)](https://github.com/xsyetopz/versionlens-redux/actions/workflows/vscode.yml)
[![Zed CI/CD](https://github.com/xsyetopz/versionlens-redux/actions/workflows/zed.yml/badge.svg)](https://github.com/xsyetopz/versionlens-redux/actions/workflows/zed.yml)
[![JetBrains CI/CD](https://github.com/xsyetopz/versionlens-redux/actions/workflows/jetbrains.yml/badge.svg)](https://github.com/xsyetopz/versionlens-redux/actions/workflows/jetbrains.yml)
[![License: ISC](https://img.shields.io/badge/license-ISC-blue.svg)](LICENSE)

## Status

This repository is a fork of VersionLens with a Rust-first implementation and a new extension identity: `xsyetopz.versionlens-redux` for VS Code. The original VS Code extension id is `pflannery.vscode-versionlens`; the VS Code adapter warns when that extension is also installed.

## Repository layout

| Path | Purpose |
| --- | --- |
| `crates/` | Rust workspace crates for parsing, provider selection, version semantics, suggestions, text edits, HTTP, N-API, and LSP. |
| `packages/vscode-extension/` | VS Code extension package, manifest, TypeScript adapter, native module packaging, and marketplace README. |
| `packages/zed-extension/` | Zed extension that starts the shared `versionlens-lsp` server. |
| `packages/jetbrains-plugin/` | IntelliJ Platform plugin that starts the shared `versionlens-lsp` server. |
| `assets/versionlens/` | Shared icons, logo, and README media reused by editor packages. |
| `scripts/` | Repository checks, build scripts, parity checks, and packaging guards. |
| `tests/` | Repo-level boundary, parser fixture, N-API, and e2e tests; VS Code unit tests live beside adapter source under `packages/vscode-extension/src/extension/__tests__/`. |

## Editor packages

| Package | README | Runtime boundary |
| --- | --- | --- |
| VS Code | [`packages/vscode-extension/README.md`](packages/vscode-extension/README.md) | TypeScript adapter + `versionlens-napi` native module |
| Zed | [`packages/zed-extension/README.md`](packages/zed-extension/README.md) | Shared `versionlens-lsp` binary |
| JetBrains | [`packages/jetbrains-plugin/README.md`](packages/jetbrains-plugin/README.md) | Shared `versionlens-lsp` binary through the IntelliJ Platform LSP API |

## Requirements

- Bun for JavaScript and TypeScript scripts.
- Rust from [`rust-toolchain.toml`](rust-toolchain.toml).
- Cargo workspace commands run from the repository root.
- Gradle for the JetBrains plugin package.

## Setup

```bash
bun install --frozen-lockfile
```

## Common commands

```bash
bun run build                 # build the VS Code extension bundle
bun run native:build          # build the Rust N-API module used by VS Code
cargo build -p versionlens-lsp # build the shared LSP server
bun run typecheck             # TypeScript checks
cargo test --workspace        # Rust tests
bun run package               # build a local VSIX
bun run check                 # full repository validation
```

`bun run check` covers Rust, TypeScript, package parity, workspace layout, adapter boundaries, VSIX freshness, source layout, and native tests. Use targeted scripts while iterating, then run the full check before committing broad changes.

## Crates

Each crate has a local `README.md` and inherits repository metadata from the workspace manifest. Crates inherit the root [`ISC`](LICENSE) license through `license.workspace = true`; do not add per-crate license files.

## Asset policy

Shared VersionLens media lives in [`assets/versionlens/`](assets/versionlens/). The VS Code package keeps `packages/vscode-extension/images` as a package-local mirror so marketplace paths such as `images/logo.png` and `images/faq/show-releases.gif` resolve inside the extension package. `bun run check:assets` verifies that the mirror matches the shared source.

## Contributing

Read [`AGENTS.md`](AGENTS.md) and [`CONTRIBUTING.md`](CONTRIBUTING.md) before changing code. Use the repository scripts in `package.json`; do not add npm, pnpm, yarn, or root Node package-manager files.

## License

[ISC](LICENSE)

## Attribution

VersionLens Redux is a fork of the original VersionLens extension by Peter Flannery and contributors. This fork keeps attribution to the original project, reuses VersionLens visual assets with attribution, and publishes under the separate `xsyetopz.versionlens-redux` VS Code identity.
