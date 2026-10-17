# Proposal

## Why

VersionLens Redux 0.4.x has two engines behind four editors. VS Code calls Rust through N-API, while JetBrains, Neovim and Zed spawn the `versionlens-lsp` binary. Because of that split, fixes land unevenly, every release builds 27 native artifacts, and the core engine depends on VS Code-specific types. Users have also reported a set of suggestion bugs, spurious red errors and missing file types. Release 0.5.0 makes the Rust LSP server the only engine, fixes the reported behavior, and cuts the codebase to what that scope needs.

## What Changes

- **BREAKING (internal):** VS Code stops loading `versionlens_napi.node`. It becomes an LSP client of the same `versionlens-lsp` binary used by JetBrains, Neovim and Zed. The `versionlens-napi` and `versionlens-vscode-model` crates are removed. User-facing commands and settings keys are unchanged. The VS Code engine floor rises from `>=1.75.0` to `^1.91.0`.
- Updates become lint-style diagnostics with code `update-available` and default severity Information, plus a "Bump to x.y.z" quick-fix code action. Every LSP editor shows them in its problems list.
- Suggestion policy:
  - Do not offer a bump that would not change the requirement text. This removes the duplicate `latest v9 | ↑ latest v9` and `=0.7.0` cases.
  - Offer a bounded set of bumps (patch, minor, major, latest) instead of one entry per intermediate version.
  - Restore the floor bump for ranges that already satisfy latest (`^2.5.8` → `^2.5.14`).
- Error reporting:
  - Skip unresolvable or unsupported inputs silently: GitHub Actions `${{ }}` expressions, unsupported setup-dotnet syntax, runtimes with no release source such as `engines.vscode`, and non-version action refs.
  - Stop publishing cancellations and internal failures as diagnostics.
  - Handle GitHub 403 and rate limits with authentication and a clear, non-blocking message.
- File types and catalogs:
  - Resolve Gradle `libs.versions.toml` `[versions]` entries through the libraries and plugins that reference them.
  - Add Cartfile, `dotnet-tools.json` and Cargo profile/config files.
  - Add file types from the same model: one descriptor per format-based file type.
- Clicking a bump in VS Code always applies the edit or reports why it could not.
- Performance: editing a manifest no longer discards resolved state or restarts workspace discovery. Heavy work runs off the LSP main loop.
- Distribution:
  - Build one `versionlens-lsp` matrix of 9 targets per release (replaces 27 builds) and publish it with SHA-256 checksums.
  - All four editors bundle or download that same binary. Neovim downloads a verified binary when none is present.
- Tooling: replace `tsc` and the `typescript` dev dependency with `bun check` (Bun 1.4.3). Rename the root `check` script, which shadows `bun check`, to `verify`.
- Codebase reduction:
  - Merge crates: versions → model, suggestions + edits → update, cache + http → fetch.
  - Use a single span-preserving parse layer and a declarative manifest table.
  - Generate per-ecosystem `package.json` settings and editor file lists from that table.
  - Remove duplicated helpers and the file-fragmentation rules that forced micro-files.
- Release as 0.5.0 across all editor packages.

## Capabilities

### New Capabilities

- `language-server`: the single engine every editor talks to. Covers protocol surface, document lifecycle, per-folder configuration, status reporting, commands, cancellation and error-reporting rules.
- `update-diagnostics`: lint-style update-available diagnostics and their quick-fix code actions.
- `version-suggestions`: which version suggestions are offered for a dependency requirement and how they are labeled.
- `manifest-support`: which files are recognized, how dependencies and version references are extracted, and how unresolvable inputs are treated.
- `registry-access`: authenticated registry requests, credential sourcing, and rate-limit and authorization failure handling.
- `editor-distribution`: how the server binary is built, verified and delivered to VS Code, JetBrains, Neovim and Zed.

### Modified Capabilities

None. `openspec/specs/` is empty. This change establishes the first specs.

## Impact

- **Rust:**
  - All crates under `crates/`.
  - `versionlens-napi` and `versionlens-vscode-model` are deleted.
  - Crates are merged as listed above.
  - Most changes are in `versionlens-core` and `versionlens-lsp`.
- **VS Code extension (`packages/vscode-extension`):**
  - Rewritten as an LSP client.
  - New dependency: `vscode-languageclient@8.1.x`, bundled by `scripts/package/extension.mjs`.
  - About 7k lines of mock-based tests are rewritten or deleted.
- **Other editors:**
  - JetBrains Gradle consumes a prebuilt binary.
  - Zed points at the shared release asset.
  - Neovim gains a verified downloader.
  - File lists are generated.
- **CI and release:**
  - A new reusable `build-lsp` workflow.
  - `vscode.yml`, `zed.yml`, `neovim.yml`, `jetbrains.yml`, `release.yml` and `publish-marketplaces.yml` are repointed to it.
  - Checksums are added.
- **Tooling:** `package.json` scripts, `tsconfig*.json`, `bun.lock`, `justfile`, `lefthook.yml`, removal of `scripts/architecture_audit`, `scripts/providers*`, `scripts/check/*` and jscpd, and docs (`AGENTS.md`, `CONTRIBUTING.md`, `README.md`, `CHANGELOG.md`).
