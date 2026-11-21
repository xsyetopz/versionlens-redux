# AGENTS.md

Repository instructions for coding agents working on this VersionLens fork.

## Scope

This repository is a Rust-first rewrite of the VersionLens VS Code extension. One language server serves every editor, and each editor package is a thin client for it:

- Language server: `crates/versionlens-lsp`, one binary crate with `pub(crate)` modules
- VS Code extension: `packages/vscode-extension/`
- JetBrains plugin: `packages/jetbrains-plugin/`, a module of the root Gradle build
- Neovim plugin: `packages/neovim-plugin/`
- Zed extension: `packages/zed-extension/`, its own Cargo workspace built for `wasm32-wasip2`
- Packaging and release scripts: `scripts/`

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

Full validation is expensive. It runs the TypeScript, Rust, and JVM checks, then `project-verify` in `project.just`: `test:release`, `check:extension-build`, `test:extension`, `package:editors`, and the Neovim tests:

```bash
just verify
```

`package:editors` builds every editor package for the host, including the JetBrains zip through `just jvm::gradle`. Neovim checks: `make lint` and `make test` in `packages/neovim-plugin`. Zed checks: `cargo test --manifest-path packages/zed-extension/Cargo.toml`.

Add a file type with one `row` in `FILE_TYPES` in `crates/versionlens-lsp/src/catalog/rows.rs`: its globs, its format (`Json`, `Toml`, `Yaml`, `Xml`, or `Text`), its registry id, and the rules that find dependencies. A new registry is one row in `REGISTRIES` in `crates/versionlens-lsp/src/registry.rs`. Editors read the globs at runtime, from `versionlens-lsp --file-types` or from `capabilities.experimental.fileTypes` in the initialize result, so no editor package changes. Zed selects files by language, so a file type in a language that `packages/zed-extension/extension.toml` lacks needs that language added there.

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
- Put project recipes in `project.just`. The `justfile` and `.just/` are shared and get replaced on the next adopt. A recipe there named `project-<task>`, such as `project-verify` or `project-test`, runs after the languages' `<task>`, so a project extends `just verify` there. `project-pre-typecheck` runs before the TypeScript type check, for a build the types import.
- Write commit messages as Conventional Commits; the commit-msg hook checks them.
- When a recipe reports a missing tool, run `just setup`, which installs the pinned ones. Install any it can't, such as `just`, Bun, rustup, or Xcode, with the system's package manager, then rerun the recipe. Tell the user what you installed, and keep the check instead of skipping it.
<!-- /bedrock -->

<!-- typescript-bedrock -->
- Install with `bun install`, and run one-off tools with `bunx`.
- Check each change with the narrowest command: `bun test <file>`, `bun check`, `just typescript::lint <path>`, or `bun fmt`. A lint run starts type-aware tsgolint, which holds the whole program in memory, so run it through `just` (one run on the machine at a time) and not as a bare `bunx oxlint`.
- Fix the cause of each lint or type error in the code, and keep the lint and `tsconfig` settings as they are.
- Pin each dependency version once, in `workspaces.catalog`, and reference it as `catalog:`.
- Give a JSDoc comment to each declaration a package exports from its `exports`, `main`, or `module` entry; internal code needs none.
- Use Bun APIs where Bun has one, such as `Bun.file`, `Bun.$`, and `Bun.spawn`.
- Change the shared lint, type, and Markdown rules in bedrock's `templates/typescript`; this repo's configs are copies that `bedrock update` rewrites.
<!-- /typescript-bedrock -->

<!-- rust-bedrock -->
- Keep Rust unit tests out of the file they test: declare `#[cfg(test)] mod tests;` with no `#[path]`, and put the body in `foo/tests.rs` beside `foo.rs`, or in `tests.rs` beside `lib.rs`, `main.rs`, or `mod.rs`. `bedrock guard rust` rejects a `#[cfg(test)] mod name { … }` with its body in the file, and `just lint-fix` moves it to `name.rs` in that same place, such as `foo/test_support.rs`. A file that only compiles under test, such as `tests.rs` and the modules below it, may hold `#[cfg(test)]` modules of its own.
- Silence a lint only with `#[expect(lint, reason = "…")]`. Clippy rejects `#[allow]`, and an `#[expect]` that no longer fires.
- Start each integration test in `tests/` with `#![cfg(test)]`, and give its helpers `pub(crate)`.
- Name module files `foo.rs` beside `foo/`, not `foo/mod.rs`.
- Document each item the crate exports, with a `# Panics` section where it can panic and `# Errors` where it returns `Result`. `pub(crate)` and private items need no docs; `just lint` runs `cargo doc`, which rejects public docs that link to private items.
- Cargo compiles through kache, the build cache `just setup` installs and turns on. When a build is slower than it should be, `kache stats --last-build` and `kache explain` show which crates missed the cache and why.
<!-- /rust-bedrock -->

<!-- jvm-bedrock -->
- Build Java and Kotlin with `./gradlew` through `just`. Each module's `build.gradle.kts` applies one `bedrock.*` plugin: `bedrock.java`, `bedrock.kotlin`, `bedrock.application`, `bedrock.native-image`, `bedrock.kmp`, `bedrock.cmp`, `bedrock.android`, or `bedrock.kotlin-native`. They compile with GraalVM Community 25 to Java 8 bytecode, with warnings as errors.
- Keep main code on the Java 8 API and Android 5 (API 21). The compilers reject newer JDK APIs, and Android Lint rejects newer Android APIs.
- Pin each dependency once, in `gradle/libs.versions.toml`, and reference it as `libs.…`. `gradle/bedrock.versions.toml` and `gradle/bedrock/` belong to bedrock, and `just guard` reports a change to them.
- Mark nullable Java types with JSpecify's `@Nullable`; NullAway treats the rest of the project's packages as non-null.
- Silence a warning only with `@SuppressWarnings` or `@Suppress` and a `//` comment on it or the line above that says why.
- Write tests with JUnit 6, `org.junit.jupiter.api`, or `kotlin.test`. JUnit 4 is only for Android tests.
- Keep only the classes reflection needs in a module's `proguard-rules.pro`; `just guard` rejects rules that turn shrinking, optimizing, or obfuscating off.
- Port Java to Kotlin with `just jvm::j2k <path>…`, which runs IntelliJ IDEA's converter and rewrites the files in place, instead of porting by hand. Commit first, then fix what the build reports.
<!-- /jvm-bedrock -->
