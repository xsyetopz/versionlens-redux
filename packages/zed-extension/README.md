# VersionLens Redux for Zed

The extension starts the shared `versionlens-lsp` server, which adds inlay hints, code lenses, hovers, vulnerability diagnostics, and update quick fixes to dependency manifests.

## Server

The extension runs the first server it finds, with `--stdio`:

1. `lsp.versionlens.binary.path` in Zed settings. `binary.arguments` replaces `--stdio` when set.
2. The `VERSIONLENS_LSP` environment variable of the worktree shell.
3. `versionlens-lsp` on the worktree's `PATH`.
4. The release matching the extension version, downloaded from GitHub and checked against its `SHA256SUMS`.

## Files

Zed starts language servers per language, not per file name. The extension registers the server for the languages its manifests use, such as TOML, JSON, YAML, XML, Go Mod, Ruby, and Plain Text. Zed opens files it has no language for, such as `requirements.txt` and `*.hxml`, as Plain Text. The server ignores every document that is not a manifest it reads.

## Settings

`lsp.versionlens.settings` is the same `versionlens` section VS Code uses:

```json
{
  "lsp": {
    "versionlens": {
      "settings": { "prereleases": true, "timeoutMs": 20000 }
    }
  }
}
```

The keys are those of the [VS Code settings](../vscode-extension/README.md#settings): `enabled`, `codeLens`, `inlayHints`, `prereleases`, `vulnerabilities`, `timeoutMs`, `concurrency`, `cacheMinutes`, `strictSsl`, `githubToken`, `registries`, `authorization`, `install`, `files`, `indicators`, `prereleaseTags`, `updateSeverity`, `stats`, `fileTypes`, `workspaceExclude`, and `providers` (per registry id, `cacheMinutes`, `strictSsl`, and `prereleaseTags` overrides). A change applies without a restart.

Zed shows inlay hints only when `inlay_hints.enabled` is on, and code lenses only when `"code_lens": "on"` (or `"menu"`) is in the user settings; Zed's default is `"off"`.

## Commands

Zed extensions cannot add toolbar buttons or palette commands (`zed_extension_api` 0.7.0 exports only language server, slash command, context server, docs, and debug adapter hooks), so the server offers every VS Code command as a code action in the code actions menu (`editor: toggle code actions`, `cmd-.` or `ctrl-.`), titled `VersionLens: …`:

- Update the file's dependencies to latest, major, minor, or patch, when one of them is outdated.
- Update the workspace's dependencies at each level.
- Sort dependencies, when the manifest has a sortable block.
- Install dependencies, when `install` holds a command for the manifest.
- Show or hide versions, show or hide prereleases, refresh versions, and clear the cache.

Per-dependency updates are quick fixes in the same menu, and code lenses when they are on. With `"stats": true`, a summary lens above each manifest shows its dependency, outdated, failed, and vulnerable counts, the counterpart of the VS Code title icons.

## Development

```bash
cargo test --manifest-path packages/zed-extension/Cargo.toml
cargo check --manifest-path packages/zed-extension/Cargo.toml --target wasm32-wasip2
```

## License

[ISC](../../LICENSE)
