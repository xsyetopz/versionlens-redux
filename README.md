# VersionLens

Rust-first VS Code extension for showing package version information directly in dependency manifests.

[![CI](https://github.com/xsyetopz/vscode-versionlens/actions/workflows/ci.yml/badge.svg)](https://github.com/xsyetopz/vscode-versionlens/actions/workflows/ci.yml)
[![License: ISC](https://img.shields.io/badge/license-ISC-blue.svg)](LICENSE)

## What this repository contains

This fork ports VersionLens behavior onto a Rust core with a VS Code extension adapter.

- Rust workspace crates live in `crates/`.
- The VS Code extension package lives in `packages/vscode-extension/`.
- Build and validation entrypoints live in `scripts/` and `package.json`.
- Repo-level boundary and e2e tests live in `tests/`.

The extension README for end users is in [`packages/vscode-extension/README.md`](packages/vscode-extension/README.md).

## Requirements

- Bun
- Rust toolchain from [`rust-toolchain.toml`](rust-toolchain.toml)
- VS Code-compatible extension host for extension testing

## Setup

```bash
bun install --frozen-lockfile
```

## Common commands

Build the TypeScript extension bundle:

```bash
bun run build
```

Build the native Rust N-API package:

```bash
bun run native:build
```

Run TypeScript checks:

```bash
bun run typecheck
```

Run Rust tests:

```bash
cargo test --workspace
```

Package a local VSIX:

```bash
bun run package
```

Run the full repository check:

```bash
bun run check
```

`bun run check` is broad and can be slow. Prefer targeted checks while iterating.

## CI/CD

GitHub Actions are configured in `.github/workflows/`:

- `ci.yml` runs toolchain, formatting, layout, adapter, type, Rust, package, and VSIX freshness checks.
- `release.yml` builds a VSIX and attaches it to GitHub releases for `v*` tags or manual dispatches.

## Contributing

Read [`CONTRIBUTING.md`](CONTRIBUTING.md) before opening a change. Use Conventional Commits and include validation results in pull requests.

## AI / Coding Agents

Agent instructions live in [`AGENTS.md`](AGENTS.md). `CLAUDE.md` and `GEMINI.md` are symlinks to the same file so agent tools read one shared repository contract.

## Code of conduct

Participation is covered by [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md), adapted from Rust and TypeScript/Microsoft open source community expectations.

## License

[`ISC`](LICENSE)
