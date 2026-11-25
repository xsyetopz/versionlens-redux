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
  ascii = false,        -- ASCII glyphs in the status component, lenses, and hints
  glyphs = {},          -- override single glyphs, e.g. { outdated = "U" }
  winbar = false,       -- show the status in each manifest window's winbar
  settings = {          -- the VS Code `versionlens.*` settings, without the prefix
    prereleases = false,
    vulnerabilities = true,
    timeoutMs = 10000,
    registries = { npm = "https://npm.example.com/{name}" },
    authorization = { ["https://npm.example.com/"] = "Bearer TOKEN" },
    install = { npm = "bun install" },
    files = { ["deps/*.list"] = "requirements" },
  },
  on_attach = nil,      -- function(client, bufnr)
})
```

Settings: booleans `enabled`, `codeLens`, `inlayHints`, `prereleases`, `vulnerabilities`, `strictSsl`, `stats`; numbers `timeoutMs`, `concurrency`, `cacheMinutes`; strings `githubToken`, `updateSeverity` (`off`, `hint`, `info`, `warning`, `error`); lists `prereleaseTags`, `fileTypes`, `workspaceExclude`; maps `registries` (registry id to URL template), `authorization` (URL prefix to Authorization header), `install` (file type or registry id to shell command, run when a save changes the dependencies), `files` (extra glob to file type id), `indicators` (glyph per state: `latest`, `update`, `checking`, `failed`, `unreadable`, `vulnerable`, `newer`, `satisfies`, `noMatch`, `directory`, `build`, `yanked`, `deprecated`), `providers` (registry id to a table of `cacheMinutes`, `strictSsl`, `prereleaseTags` overriding the global value, such as `providers = { npm = { cacheMinutes = 5 } }`). Unknown keys and wrong types are reported by `setup()`. `githubToken` falls back to `$GITHUB_TOKEN` and `$GH_TOKEN`. Calling `setup()` again sends the new settings to running servers, with no restart. `:VersionLens settings` opens the settings page, a Lua buffer listing every setting with its value and description; `:w` applies it to running servers and keeps what differs from `setup()` in `stdpath("data")/versionlens/settings.json` (mode 0600).

## Commands

`:VersionLens` with no argument opens a `vim.ui.select` menu of what applies to the current buffer (show or hide, prereleases, update levels while dependencies are outdated, sort, install, refresh, workspace update, and more). `:VersionLens <subcommand>` completes with `<Tab>`:

- `show`, `hide`, `toggle`: show or hide versions in the buffer.
- `prereleases [show|hide|toggle]`: prerelease versions; default `toggle`.
- `update [latest|major|minor|patch]`: update every dependency in the buffer; default `latest`.
- `workspace [level]`: update every manifest in the workspace.
- `sort`: sort the manifest's dependencies.
- `install`: run the manifest's install command.
- `refresh`: fetch the buffer's versions again.
- `clear-cache`: drop cached registry answers.
- `log`: show the server's log messages.
- `restart`: restart the server.
- `info`: show the server command and the buffer's status.
- `settings`: open the settings page described above, like VS Code's settings page for the `versionlens` section.
- `auth [add|remove]`: store or forget the Authorization header for a registry URL prefix, like VS Code's "Add url authentication" and "Remove url authentication"; default `add`. Neovim has no secret store, so the headers live in `stdpath("data")/versionlens/authorization.json`, readable only by its owner (mode 0600). A stored header replaces an `authorization` setting for the same prefix.

`updateAll`, `clearCache`, and `output` still work as aliases. Per-dependency updates are code actions (`vim.lsp.buf.code_action()`) and code lenses (`vim.lsp.codelens.run()`). `:checkhealth versionlens` checks the server.

## Status Line

The server reports each manifest's state with `versionlens/status`. `require("versionlens").statusline()` renders it for the current buffer, for example `◉ ↑3 ⚠1 β`:

| Unicode | ASCII | Meaning |
| --- | --- | --- |
| `◉` | `VL` | versions shown |
| `◌` | `vl-` | versions hidden |
| `⠋` ... | `\|/-` | working (spinner) |
| `↑N` | `^N` | N dependencies outdated |
| `✗N` | `xN` | N lookups failed |
| `⚠N` | `!N` | N vulnerable |
| `✓` | `ok` | all up to date |
| `β` | `pre` | prereleases on |

```lua
-- lualine
sections = { lualine_x = { require("versionlens").statusline } }
-- built-in statusline or winbar
vim.o.winbar = "%{%v:lua.require'versionlens'.statusline()%}" -- or setup({ winbar = true })
```

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
