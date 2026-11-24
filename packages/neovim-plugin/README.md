# VersionLens Redux for Neovim

Starts the shared Rust `versionlens-lsp` server on every manifest it reads and shows the latest versions as code lenses, inlay hints, and diagnostics, with code actions to update.

## Requirements

- Neovim 0.11 or newer.
- `curl` and `tar` for the automatic server download.

## Installation

Extract the release archive for your platform as a Neovim package; it carries the matching server in `bin/`:

```bash
dir="${XDG_DATA_HOME:-$HOME/.local/share}/nvim/site/pack/versionlens/start/versionlens-redux"
mkdir -p "$dir" && tar -xzf versionlens-redux-neovim-plugin-linux-x64.tar.gz -C "$dir"
```

A plugin manager that clones the repository works too; the plugin then downloads the server on first use.

## Server

The plugin uses the first server it finds:

1. The `cmd` option.
2. `$VERSIONLENS_LSP`.
3. `bin/versionlens-lsp` on `runtimepath` (the release archive).
4. `versionlens-lsp` on `PATH`.
5. A previous download in `stdpath("data")/versionlens/<version>/`.

With none found and `auto_download = true`, it downloads `versionlens-lsp-<version>-<triple>` from the GitHub release, checks it against the release `SHA256SUMS`, and installs it in that data directory. A failed download or checksum installs nothing and is reported with `vim.notify`.

The plugin asks the server which files it reads (`versionlens-lsp --file-types`), so every manifest the server supports attaches without plugin changes.

## Setup

The plugin calls `setup()` on load. Call it again to change options, or set `vim.g.versionlens_disable_auto_setup = 1` to call it yourself:

```lua
require("versionlens").setup({
  cmd = nil,            -- { "/path/to/versionlens-lsp" }
  auto_download = true,
  autostart = true,
  codelens = true,
  inlay_hints = true,
  settings = {          -- the VS Code `versionlens.*` settings, without the prefix
    prereleases = false,
    vulnerabilities = true,
    timeoutMs = 10000,
    registries = { npm = "https://npm.example.com/{name}" },
  },
  on_attach = nil,      -- function(client, bufnr)
})
```

Settings: `enabled`, `codeLens`, `inlayHints`, `prereleases`, `vulnerabilities`, `timeoutMs`, `concurrency`, `cacheMinutes`, `strictSsl`, `githubToken`, `registries`. `githubToken` falls back to `$GITHUB_TOKEN` and `$GH_TOKEN`. Calling `setup()` again sends the new settings to running servers, with no restart.

## Commands

`:VersionLens <subcommand>`:

- `refresh`: fetch the current buffer's versions again.
- `updateAll`: update every dependency in the buffer to its latest version.
- `toggle`: show or hide lenses and hints.
- `clearCache`: drop cached registry answers.
- `restart`: restart the server.
- `log`: show the server's log messages.
- `info`: show the server command.

Per-dependency updates are code actions (`vim.lsp.buf.code_action()`) and code lenses (`vim.lsp.codelens.run()`). `:checkhealth versionlens` checks the server.

## Development

```bash
make -C packages/neovim-plugin lint
make -C packages/neovim-plugin test
```

`make test` builds the server and runs `tests/run.lua` headless against it, without plugins or user config.

## License

[ISC](LICENSE)

## Attribution

VersionLens Redux is a fork of the original VersionLens extension by Peter Flannery and contributors.
