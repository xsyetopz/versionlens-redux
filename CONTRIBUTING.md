# Contributing

Thanks for working on VersionLens.

## Before changing code

1. Read `README.md` for product scope.
2. Read `AGENTS.md` for repository-specific build, test, and safety rules.
3. Inspect the affected Rust crates, TypeScript adapter code, scripts, and tests before editing.

## Development setup

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

## Adding a file type

Add one row to the table in `crates/versionlens-parsers/src/manifest/table.rs` (with a `ManifestKind` variant in `crates/versionlens-model/src/manifest.rs` and a parser function, new or existing), then run `bun run generate:manifests`. `bun run check:manifests` fails when the generated editor file lists are stale.

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
bun run typecheck   # bun check --cwd packages/vscode-extension
cargo fmt --all -- --check
cargo test --workspace
bun run check:manifests
```

Before a release or broad refactor, run:

```bash
bun run verify
```

`bun run verify` is intentionally broad and can be slow, and the JetBrains build needs Java 21. Neovim tests run with `make test` in `packages/neovim-plugin`; e2e tests with `bun run test:e2e`. Do not run it in the background unless someone explicitly asked for that.

## Pull requests

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

## Code review expectations

- Keep unrelated formatting and cleanup out of behavior changes.
- Preserve command IDs, settings keys, and extension contribution points unless the change requires a migration.
- Prefer source-backed parity with upstream VersionLens behavior when changing user-visible behavior.
- Do not commit generated VSIX files, `target/`, `dist/`, `node_modules/`, staged server binaries, or TypeScript build info.
