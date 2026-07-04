# AGENTS.md

Repository instructions for coding agents working on this VersionLens fork.

## Scope

This repository is a Rust-first rewrite of the VersionLens VS Code extension. Keep product source split by boundary:

- Rust crates: `crates/`
- VS Code extension package: `packages/vscode-extension/`
- Repo-level scripts: `scripts/`
- Repo-level boundary and e2e tests: `tests/`

Do not reintroduce `_archive`, root `src/`, or prose-only generated ledgers.

## Toolchain

Use the repository toolchain and package manager:

- Bun for JavaScript/TypeScript commands.
- Rust from `rust-toolchain.toml`.
- Cargo workspace commands from the repository root.

Do not add npm, pnpm, yarn, `package-lock.json`, or root Node package managers other than Bun.

## Commands

Use the scripts already defined in `package.json`.

Small checks:

```bash
bun run check:toolchain
bun run check:workspace-layout
bun run check:vscode-adapter
bun run typecheck
cargo fmt --all -- --check
```

Package the VS Code extension:

```bash
bun run package
```

Full validation is expensive and includes Rust, TypeScript, extension, and native checks:

```bash
bun run check
```

Do not leave long-running background checks unattended. Prefer the smallest targeted validation that covers the change.

## Style and architecture

- Preserve existing behavior unless the requested task explicitly changes it.
- Inspect callsites before changing public contracts across Rust, N-API, and TypeScript boundaries.
- Keep VS Code command IDs, contributed menu contexts, and activation events aligned with `packages/vscode-extension/package.json`.
- Keep Rust/TypeScript boundary data objects explicit and covered by existing check scripts.
- Avoid wrappers, compatibility shims, new dependencies, or generated artifacts unless the repository contract requires them.

## Git

- `origin` is the GitHub fork.
- `upstream` is the external VersionLens source and must remain fetch-only.
- Use Conventional Commits for agent-authored commits.
- Add Codex co-authoring when Codex CLI materially contributes:

```text
Co-authored-by: Codex <noreply@openai.com>
```
