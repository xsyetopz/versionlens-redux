# VersionLens Redux

VersionLens Redux shows dependency version information in editor manifests using one Rust LSP server, `versionlens-lsp`, shared by the VS Code, Zed, Neovim, and JetBrains adapters.

[![VS Code CI/CD](https://github.com/xsyetopz/versionlens-redux/actions/workflows/vscode.yml/badge.svg)](https://github.com/xsyetopz/versionlens-redux/actions/workflows/vscode.yml)
[![Zed CI/CD](https://github.com/xsyetopz/versionlens-redux/actions/workflows/zed.yml/badge.svg)](https://github.com/xsyetopz/versionlens-redux/actions/workflows/zed.yml)
[![Neovim CI/CD](https://github.com/xsyetopz/versionlens-redux/actions/workflows/neovim.yml/badge.svg)](https://github.com/xsyetopz/versionlens-redux/actions/workflows/neovim.yml)
[![JetBrains CI/CD](https://github.com/xsyetopz/versionlens-redux/actions/workflows/jetbrains.yml/badge.svg)](https://github.com/xsyetopz/versionlens-redux/actions/workflows/jetbrains.yml)
[![License: ISC](https://img.shields.io/badge/license-ISC-blue.svg)](LICENSE)

## Status

This repository is a fork of VersionLens with a Rust-first implementation and a new extension identity: `xsyetopz.versionlens-redux` for VS Code. The original VS Code extension id is `pflannery.vscode-versionlens`.

## Repository Layout

| Path | Purpose |
| --- | --- |
| `crates/versionlens-lsp/` | The language server: manifest catalog, extraction, registry lookups, OSV checks, version ranking, and LSP handling, in one binary crate. |
| `packages/vscode-extension/` | VS Code extension: a TypeScript LSP client, settings, and the bundled server. |
| `packages/zed-extension/` | Zed extension that downloads and starts the server. |
| `packages/neovim-plugin/` | Neovim plugin that starts the server and exposes `:VersionLens`. |
| `packages/jetbrains-plugin/` | IntelliJ Platform plugin with a settings page, actions, and a log window. |
| `assets/versionlens/` | Shared icons, logo, and README media reused by editor packages. |
| `scripts/` | Packaging and release scripts. |

## Editor Packages

| Package | README | Runtime boundary |
| --- | --- | --- |
| VS Code | [`packages/vscode-extension/README.md`](packages/vscode-extension/README.md) | TypeScript LSP client + shared `versionlens-lsp` binary |
| Zed | [`packages/zed-extension/README.md`](packages/zed-extension/README.md) | Shared `versionlens-lsp` binary |
| Neovim | [`packages/neovim-plugin/README.md`](packages/neovim-plugin/README.md) | Lua adapter + shared `versionlens-lsp` binary |
| JetBrains | [`packages/jetbrains-plugin/README.md`](packages/jetbrains-plugin/README.md) | Shared `versionlens-lsp` binary through the IntelliJ Platform LSP API |

## Requirements

- Bun 1.4.3 for JavaScript and TypeScript scripts.
- Rust from [`rust-toolchain.toml`](rust-toolchain.toml).
- `just`. `just setup` installs the other pinned tools.
- A JDK 17 or newer to start Gradle for the JetBrains plugin. Gradle downloads the compiler JDK itself.

## Common Commands

```bash
just                          # list the recipes
just setup                    # install pinned tools and dependencies
cargo build -p versionlens-lsp # build the server
cargo test -p versionlens-lsp  # server tests
just typecheck                # TypeScript and JVM type checks
just lint                     # every linter
just package                  # build a local VSIX
bun run package:editors       # build every editor package for this machine
just verify                   # full validation
```

The release and Marketplace publication procedure is documented in [`docs/releasing.md`](docs/releasing.md). Publication is a separate, manually dispatched, approval-gated workflow and never runs for pull requests.

## Adding a File Type

Add one `[[file]]` row to the group file for its ecosystem in [`crates/versionlens-lsp/catalog/files/`](crates/versionlens-lsp/catalog/files): its globs, its format (`json`, `toml`, `yaml`, `xml`, `ini`, or `text`), its registry id, and the rules that locate dependencies. A new registry is one `[[registry]]` row in [`catalog/registries/`](crates/versionlens-lsp/catalog/registries). The server embeds these TOML files, and each one's header comment gives its row format. JetBrains and Neovim read the globs from `versionlens-lsp --file-types`, and VS Code reads them from `capabilities.experimental.fileTypes` in the initialize result, so they need no change. Zed picks files by language, so a manifest in a language that [`extension.toml`](packages/zed-extension/extension.toml) does not list needs that language added there.

## Asset Policy

Shared VersionLens media lives in [`assets/versionlens/`](assets/versionlens/). The VS Code package keeps `packages/vscode-extension/images` as a package-local mirror so marketplace paths such as `images/logo.png` and `images/faq/show-releases.gif` resolve inside the extension package.

## Contributing

Read [`AGENTS.md`](AGENTS.md) and [`CONTRIBUTING.md`](CONTRIBUTING.md) before changing code. Use the repository scripts in `package.json`; do not add npm, pnpm, yarn, or root Node package-manager files.

## License

[ISC](LICENSE)

## Attribution

VersionLens Redux is a fork of the original VersionLens extension by Peter Flannery and contributors. This fork keeps attribution to the original project, reuses VersionLens visual assets with attribution, and publishes under the separate `xsyetopz.versionlens-redux` VS Code identity.
