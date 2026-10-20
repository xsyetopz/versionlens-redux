# VersionLens Redux for Zed

The Zed extension starts the shared `versionlens-lsp` server so Zed can show VersionLens Redux dependency diagnostics and code lenses.

## Package

- Extension id: `versionlens-lsp`
- Display name: `VersionLens Redux`
- Manifest: [`extension.toml`](extension.toml)
- Runtime server: `crates/versionlens-lsp`

## Build the language server

From the repository root:

```bash
cargo build -p versionlens-lsp
```

The extension does not bundle the server. It resolves it in this order:

1. `lsp.versionlens.binary.path` in Zed settings.
2. `versionlens-lsp` on `PATH`.
3. The release archive for the extension version (`versionlens-lsp-<version>-<target>`), downloaded from GitHub, checked against the release `SHA256SUMS`, and extracted into the extension directory. Older `versionlens-lsp-*` installs are removed. Targets come from `scripts/package/targets.json` at build time (`build.rs`). Zed grants the extension narrowly scoped permission to download only VersionLens Redux GitHub release assets.

## Settings

The `lsp.versionlens.settings` object is returned to the server as its `versionlens` workspace configuration (`workspace/configuration`). Use the same keys as the VS Code `versionlens` settings (see `contributes.configuration` in [`../vscode-extension/package.json`](../vscode-extension/package.json)):

```json
{
  "lsp": {
    "versionlens": {
      "settings": {
        "diagnostics": { "updateSeverity": "warn" }
      }
    }
  }
}
```

`diagnostics.updateSeverity` sets the severity of update-available diagnostics: `error`, `warn`, `info` (default), `hint`, or `off`. `off` hides them; vulnerability diagnostics are unaffected.

## Private registries

Zed does not implement the `versionlens/authorization` request, so the server uses its other credential sources:

- For `api.github.com`: `GITHUB_TOKEN`, then `GH_TOKEN`, then the output of `gh auth token`, which is only tried after GitHub answers 401, 403, or 429.
- Registry configuration files such as `.npmrc`, `.yarnrc.yml`, and `NuGet.config`.

Start Zed with the variables set. Tokens are never written to logs, diagnostics, or the cache.

## Zed verification

Enable version checks and update commands in Zed's Code Actions menu:

```json
{
  "code_lens": "menu"
}
```

Place the cursor on a dependency and run **Editor: Code Actions** to select an available version.

Zed also supports `"code_lens": "on"` for inline display. In Zed 1.18.1, an inline lens can retain its previous command after an edit; VersionLens rejects that stale command. Use the Code Actions menu to apply the current update.

```bash
cargo check --manifest-path packages/zed-extension/Cargo.toml --locked
cargo test --manifest-path packages/zed-extension/Cargo.toml --locked
cargo build --manifest-path packages/zed-extension/Cargo.toml --release --locked
```

The repository-wide integration gate is `bun run check`.

The Zed extension code uses the registry-supported [MIT license](LICENSE). The language server and the rest of the repository remain under [ISC](../../LICENSE).

## Attribution

VersionLens Redux is a fork of the original VersionLens extension by Peter Flannery and contributors. Zed support uses the fork's shared Rust LSP server and the shared assets in [`../../assets/versionlens`](../../assets/versionlens) when editor media is needed.
