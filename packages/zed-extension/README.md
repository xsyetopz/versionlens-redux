# VersionLens Redux for Zed

The extension starts the shared `versionlens-lsp` server, which adds inlay hints, code lenses, hovers, vulnerability diagnostics, and update quick fixes to dependency manifests.

## Server

The extension runs the first server it finds, with `--stdio`:

1. `lsp.versionlens.binary.path` in Zed settings. `binary.arguments` replaces `--stdio` when set.
2. The `VERSIONLENS_LSP` environment variable of the worktree shell.
3. `versionlens-lsp` on the worktree's `PATH`.
4. The release matching the extension version, downloaded from GitHub and checked against its `SHA256SUMS`.

## Files

Zed starts language servers per language, not per file name. The extension registers the server for the languages its manifests use, such as TOML, JSON, YAML, XML, Go Mod, Ruby, and Plain Text. Zed opens `*.txt` files, such as `requirements.txt`, as Plain Text. Some languages come from Zed extensions rather than from Zed itself: `Cargo.toml` and `pyproject.toml` need the `toml` extension, and a `Dockerfile` needs the `dockerfile` extension. Zed offers to install them when such a file opens. Zed starts no language server for a file it has no language for, such as `*.hxml`. Map such files to Plain Text in the user settings:

```json
{ "file_types": { "Plain Text": ["*.hxml"] } }
```

The server ignores every document that is not a manifest it reads.

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

Zed extensions cannot register palette commands or toolbar buttons of their own (the extension API exports language server, slash command, context server, docs, and debug adapter hooks). The server therefore offers every VS Code command as a code action titled `VersionLens: …`, and Zed opens that menu three ways:

- the keyboard: `cmd-.` or `ctrl-.`;
- the command palette: `editor: toggle code actions`;
- the editor toolbar: the ⚡ Code Actions button, shown with `"toolbar": { "code_actions": true }` (Zed's default is `false`).

The menu holds:

- Update the file's dependencies to latest, major, minor, or patch, when one of them is outdated.
- Update the workspace's dependencies at each level.
- Sort dependencies, when the manifest has a sortable block.
- Install dependencies, when `install` holds a command for the manifest.
- Show or hide versions, show or hide prereleases, refresh versions, and clear the cache.
- Add URL authentication and Remove URL authentication, described below.

The remaining VS Code commands map to Zed built-ins and settings:

| VS Code | Zed |
| --- | --- |
| Restart server | `editor: restart language server` |
| Show output, show error | `dev: open language server logs`, server `versionlens` |
| Change build (`≌` lens) | Zed asks which build in a prompt and applies the choice |
| Open a local dependency (`📁` lens) | Opens the directory with the system handler |

Per-dependency updates are quick fixes in the same menu, and code lenses when they are on. With `"stats": true`, a summary lens above each manifest shows its dependency, outdated, failed, and vulnerable counts, the counterpart of the VS Code title icons.

## URL Authentication

Add URL authentication asks how the registry of the current manifest authenticates: Basic (user and password), Bearer token, or Header value. It then writes a blank entry for the registry's URL prefix to the stored authentication file and opens the file with the cursor in the blank:

```json
{
  "https://registry.npmjs.org/": "Token abc",
  "https://npm.example.com/": { "token": "" },
  "https://maven.example.com/": { "user": "", "password": "" }
}
```

A string is sent as the Authorization header as written, `token` as `Bearer <token>`, and `user` with `password` as `Basic <base64>`; an entry with a blank part sends nothing. Saving the file applies it on the server's next request, and it takes precedence over `authorization` in the settings for the same prefix. The file is `~/.config/versionlens/authorization.json` (`$XDG_CONFIG_HOME` when set; `%APPDATA%\versionlens\authorization.json` on Windows), written with mode 0600. Remove URL authentication asks which stored prefix to forget and removes it. Zed's prompts take buttons but no typed text, so the value is typed into the file instead of a prompt.

## Development

```bash
cargo test --manifest-path packages/zed-extension/Cargo.toml
cargo check --manifest-path packages/zed-extension/Cargo.toml --target wasm32-wasip2
```

## License

[ISC](../../LICENSE)
