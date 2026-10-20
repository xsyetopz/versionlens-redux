# AGENTS.md

Repository instructions for coding agents working on this VersionLens fork.

## Scope

This repository is a Rust-first rewrite of the VersionLens VS Code extension. Keep product source split by boundary:

- Rust crates: `crates/` (`versionlens-engine`, `-fetch`, `-lsp`, `-model`, `-parsers`, `-providers`, `-update`, and `-test-support`, a dev-dependency only)
- VS Code extension package: `packages/vscode-extension/`
- Repo-level scripts: `scripts/`
- Repo-level boundary and e2e tests: `tests/`

Do not reintroduce `_archive`, root `src/`, or prose-only generated ledgers.

## Toolchain

Use the repository toolchain and package manager:

- Bun 1.4.3 (`packageManager` in `package.json`) for JavaScript/TypeScript commands.
- Rust from `rust-toolchain.toml`.
- Cargo workspace commands from the repository root.

Do not add npm, pnpm, yarn, `package-lock.json`, or root Node package managers other than Bun.

## Commands

Use the scripts already defined in `package.json`.

Small checks:

```bash
bun run typecheck   # runs `bun check --cwd packages/vscode-extension`
cargo fmt --all -- --check
```

Package the VS Code extension:

```bash
bun run package
```

Full validation is expensive and chains `check:biome`, `check:rustfmt`, `check:manifests`, `check:manifest-baseline`, `test:release`, `check:clippy`, `typecheck`, `check:extension-build`, `cargo test --workspace --locked`, `test:extension`, and `package:editors`:

```bash
bun run verify
```

`verify` builds the JetBrains plugin, which needs Java 21 (for example `JAVA_HOME` pointing at an openjdk@21 install). Neovim tests: `make test` in `packages/neovim-plugin`. E2E: `bun run test:e2e`.

Add a file type with one row in `crates/versionlens-parsers/src/manifest/table.rs` (plus its `ManifestKind` variant in `crates/versionlens-model/src/manifest.rs` and a parser function, new or existing), then run `bun run generate:manifests` to refresh the editor file lists. `bun run check:manifests` verifies they are current.

Do not leave long-running background checks unattended. Prefer the smallest targeted validation that covers the change.

## Style and architecture

- Preserve existing behavior unless the requested task explicitly changes it.
- Inspect callsites before changing public contracts across the Rust LSP server and the editor adapters.
- Keep VS Code command IDs, contributed menu contexts, and activation events aligned with `packages/vscode-extension/package.json`.
- Keep LSP protocol data explicit; every editor talks to the one `versionlens-lsp` server.
- Avoid wrappers, compatibility shims, new dependencies, or generated artifacts unless the repository contract requires them.

## Git

- `origin` is the GitHub fork.
- `upstream` is the external VersionLens source and must remain fetch-only.
- Use Conventional Commits for agent-authored commits.
- Add Codex co-authoring when Codex CLI materially contributes:

```text
Co-authored-by: Codex <noreply@openai.com>
```
