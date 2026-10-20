# Proposal

## Why

Release 0.5.0 shipped with five items deferred from `unify-lsp-and-fix-suggestions`:

- The code is larger than it needs to be. Product Rust is about 47.3k lines and VS Code TypeScript about 2.0k. The earlier 27k / 1.2k figures were speculative estimates, not fixed goals.
- Test HTTP servers use a blocking `accept()` followed by `join()`. If a client never connects, the test hangs instead of failing. Only one site is fixed (`c41fbee7`, task 9.4).
- CI never runs `check:manifests`, `check:manifest-baseline` or `test:e2e`. Each push to master and each pull request runs `build-lsp` three times, once from each of the VS Code, JetBrains and Neovim workflows, 27 native builds in all.
- `docs/workspace-source-of-truth.md` still says "(0.4.0)" in its heading and table header.
- The `editor-distribution` spec has a requirement over 500 characters, so `openspec validate --specs --strict` fails.

## What Changes

- Ship all of the fixes below as release 0.5.1.
- Cut product Rust to about 40k lines or fewer and VS Code TypeScript to about 1.8k or fewer, by removing duplication and dead paths. No behavior change. Targets and reasoning are in design.md.
- Replace blocking `accept()` in test servers with one bounded accept helper in `versionlens-test-support`, and use it at every site.
- Add one `ci.yml` workflow that builds the language server once and calls the editor workflows with that artifact. Add `check:manifests`, `check:manifest-baseline` and `test:e2e` to it.
- Fix the stale "0.4.0" labels in the workspace doc.
- Split the "JetBrains and Zed use release artifacts" requirement into two requirements, one per editor.

## Capabilities

### Modified Capabilities

- `editor-distribution`: the JetBrains and Zed requirement becomes two shorter requirements. No behavior changes.

## Impact

- Rust crates: `parsers`, `engine`, `update`, `fetch`, `lsp`, `providers` (refactor only), plus test-only edits in `test-support`.
- `packages/vscode-extension/src/extension/auth/` (helper consolidation).
- `.github/workflows/`: new `ci.yml`; `vscode.yml`, `jetbrains.yml`, `neovim.yml`, `zed.yml` lose their `push` and `pull_request` triggers.
- `docs/workspace-source-of-truth.md`, `openspec/specs/editor-distribution`.
- No public contract changes: LSP protocol, command IDs, settings keys and snapshots stay as they are.
