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

Use the `just` recipes; `just` lists them, and `project.just` holds the VersionLens ones.

Small checks:

```bash
just typecheck
just rust::format-check
```

Package the VS Code extension:

```bash
just package
```

Full validation is expensive. It runs the TypeScript and Rust checks, then `project-verify` in `project.just`: `check:manifests`, `test:release`, `check:extension-build`, `test:extension`, and `package:editors`:

```bash
just verify
```

`project-verify` builds the JetBrains plugin, which needs Java 21 (for example `JAVA_HOME` pointing at an openjdk@21 install). Neovim tests: `make test` in `packages/neovim-plugin`. E2E: `bun run test:e2e`.

Add a file type with one row in `crates/versionlens-parsers/src/manifest/table.rs` (plus its `ManifestKind` variant in `crates/versionlens-model/src/manifest.rs` and a parser function, new or existing), then run `bun run generate:manifests` to refresh the editor file lists. `bun run check:manifests` verifies they are current.

Do not leave long-running background checks unattended. Prefer the smallest targeted validation that covers the change.

## Style and Architecture

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

<!-- bedrock -->
- Run each task through `just`. `just` lists the recipes. Check each change with the narrowest one, such as `just rust::test` or `just lint`, and run `just verify` once before you finish a change that alters behavior.
- Fix the cause of each lint, type, or guard error in the code. Keep the shared configs as they are; the bedrock that wrote them owns them.
- Put project recipes in `project.just`. The `justfile` and `.just/` are shared and get replaced on the next adopt. A recipe there named `project-<task>`, such as `project-verify` or `project-test`, runs after the languages' `<task>`, so a project extends `just verify` there.
- Write commit messages as Conventional Commits; the commit-msg hook checks them.
- When a recipe reports a missing tool, run `just setup`, which installs the pinned ones. Install any it can't, such as `just`, Bun, rustup, or Xcode, with the system's package manager, then rerun the recipe. Tell the user what you installed, and keep the check instead of skipping it.
<!-- /bedrock -->

<!-- typescript-bedrock -->
- Install with `bun install`, and run one-off tools with `bunx`.
- Check each change with the narrowest command: `bun test <file>`, `bun check`, `bun run lint`, or `bun fmt`.
- Fix the cause of each lint or type error in the code, and keep the lint and `tsconfig` settings as they are.
- Pin each dependency version once, in `workspaces.catalog`, and reference it as `catalog:`.
- Give a JSDoc comment to each declaration a package exports from its `exports`, `main`, or `module` entry; internal code needs none.
- Use Bun APIs where Bun has one, such as `Bun.file`, `Bun.$`, and `Bun.spawn`.
- Change shared lint, type, and Markdown rules in `typescript-bedrock`, which the repo configs extend.
<!-- /typescript-bedrock -->

<!-- rust-bedrock -->
- Keep Rust unit tests inline, as `#[cfg(test)] mod tests { … }` at the end of the file. `cargo bedrock guard` rejects a `mod tests;` whose body sits in another file, and `just lint-fix` moves it inline.
- Silence a lint only with `#[expect(lint, reason = "…")]`. Clippy rejects `#[allow]`, and an `#[expect]` that no longer fires.
- Start each integration test in `tests/` with `#![cfg(test)]`, and give its helpers `pub(crate)`.
- Name module files `foo.rs` beside `foo/`, not `foo/mod.rs`.
- Document each item the crate exports, with a `# Panics` section where it can panic and `# Errors` where it returns `Result`. `pub(crate)` and private items need no docs; `just lint` runs `cargo doc`, which rejects public docs that link to private items.
- Cargo compiles through kache, the build cache `just setup` installs and turns on. When a build is slower than it should be, `kache stats --last-build` and `kache explain` show which crates missed the cache and why.
<!-- /rust-bedrock -->
