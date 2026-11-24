# Contributing

Thanks for working on VersionLens.

## Before Changing Code

1. Read `README.md` for product scope.
2. Read `AGENTS.md` for repository-specific build, test, and safety rules.
3. Inspect the affected Rust crates, TypeScript adapter code, scripts, and tests before editing.

## Development Setup

Install Bun 1.4.3 and the Rust toolchain declared in `rust-toolchain.toml`, then install dependencies:

```bash
bun install --frozen-lockfile
```

Build the VS Code extension bundle:

```bash
bun run build
```

Package a local VSIX:

```bash
bun run package
```

Build all editor packages and verify their bundled runtimes:

```bash
bun run package:editors
```

## Adding a File Type

Add one `row` to `FILE_TYPES` in `crates/versionlens-lsp/src/catalog/rows.rs`, and a `REGISTRIES` row in `crates/versionlens-lsp/src/registry.rs` when the registry is new. Editors read the globs from `versionlens-lsp --file-types` or from `capabilities.experimental.fileTypes` in the initialize result, so their packages stay unchanged; Zed needs the file's language in `packages/zed-extension/extension.toml`.

## Release

1. Bump and commit the synchronized repository version:

```bash
bun run version:bump 0.2.0
```

1. Push the commit to `master` and wait for the VS Code, Zed, and JetBrains workflows to pass.
2. Run the **Release** workflow from `master` with the same version.

The release workflow refuses stale commits, mismatched versions, existing tags, and commits without successful editor workflows. It downloads the compiled artifacts from those exact workflow runs before creating the tag and GitHub release.

## Validation

Run the smallest check that covers your change. Common checks:

```bash
cargo test -p versionlens-lsp
bun test packages/vscode-extension
just jvm::verify
make -C packages/neovim-plugin test
```

Before a release or broad refactor, run:

```bash
just verify
```

`just verify` is broad and slow: it builds every editor package. Do not run it in the background unless someone explicitly asked for that.

`just vscode-live` packages the extension and drives it in an isolated VS Code window against the live registries and OSV: hints, lenses, quick fixes, hovers, diagnostics, settings, every palette command, restart, and the Output log. It needs a desktop session and the network, so `just verify` leaves it out. Set `VSCODE_BIN` when VS Code is not at the macOS default path.

`just jetbrains-live` builds the server and runs `LiveTest` in a headless IntelliJ IDEA, using the plugin's own client against the live registries and OSV: hints, diagnostics, the update intention, hover, settings, every action, and restart. It needs the network and no desktop. Other test runs skip `LiveTest`.

`just neovim-live` builds the server and runs `packages/neovim-plugin/tests/live.lua` in a headless Neovim against the live registries and OSV: hints, advisories, hover, the lens that opens the package page, the update code action, every `:VersionLens` subcommand, settings pushed by `setup()`, restart, and `libraries.hxml`. It needs the network, so `just verify` runs only the offline `tests/run.lua`.

## Pull Requests

A good pull request includes:

- a narrow description of the behavior changed;
- the files or subsystems touched;
- validation commands run and their results;
- any behavior left unverified.

Use Conventional Commits for commit messages, for example:

```text
fix(extension): stop refresh loop after toggling lenses
```

If OpenAI Codex CLI materially contributed, include:

```text
Co-authored-by: Codex <noreply@openai.com>
```

## Code Review Expectations

- Keep unrelated formatting and cleanup out of behavior changes.
- Preserve command IDs, settings keys, and extension contribution points unless the change requires a migration.
- Prefer source-backed parity with upstream VersionLens behavior when changing user-visible behavior.
- Do not commit generated VSIX files, `target/`, `dist/`, `node_modules/`, staged server binaries, or TypeScript build info.
