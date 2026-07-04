# Contributing

Thanks for working on VersionLens.

## Before changing code

1. Read `README.md` for product scope.
2. Read `AGENTS.md` for repository-specific build, test, and safety rules.
3. Inspect the affected Rust crates, TypeScript adapter code, scripts, and tests before editing.

## Development setup

Install Bun and the Rust toolchain declared in `rust-toolchain.toml`, then install dependencies:

```bash
bun install --frozen-lockfile
```

Build the extension:

```bash
bun run build
```

Package a local VSIX:

```bash
bun run package
```

## Validation

Run the smallest check that covers your change. Common checks:

```bash
bun run check:toolchain
bun run check:workspace-layout
bun run check:vscode-adapter
bun run typecheck
cargo fmt --all -- --check
cargo test --workspace
```

Before a release or broad refactor, run:

```bash
bun run check
```

`bun run check` is intentionally broad and can be slow. Do not run it in the background unless someone explicitly asked for that.

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
- Do not commit generated VSIX files, `target/`, `dist/`, `node_modules/`, native `.node` outputs, or TypeScript build info.
