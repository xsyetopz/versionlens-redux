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

The keys are `enabled`, `codeLens`, `inlayHints`, `prereleases`, `vulnerabilities`, `timeoutMs`, `concurrency`, `cacheMinutes`, `strictSsl`, `githubToken`, and `registries`. The [VS Code README](../vscode-extension/README.md#settings) describes each one. A change applies without a restart.

Zed shows inlay hints only when `inlay_hints.enabled` is on. The update quick fixes are in the code actions menu.

## Development

```bash
cargo test --manifest-path packages/zed-extension/Cargo.toml
cargo check --manifest-path packages/zed-extension/Cargo.toml --target wasm32-wasip2
```

## License

[ISC](../../LICENSE)
