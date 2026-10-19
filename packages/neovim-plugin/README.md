# VersionLens Redux for Neovim

The Neovim plugin starts the shared Rust `versionlens-lsp` server and renders dependency diagnostics and code lenses in supported manifests.

## Requirements

- Neovim 0.10 or newer.
- A release archive with its bundled server, `versionlens-lsp` on `PATH`, `VERSIONLENS_LSP`, or an explicit `cmd` option.

## Installation

Download the release archive matching your platform and architecture, then extract it as a normal Neovim package. For example:

```bash
install_dir="${XDG_DATA_HOME:-$HOME/.local/share}/nvim/site/pack/versionlens/start/versionlens-redux"
mkdir -p "$install_dir"
tar -xzf versionlens-redux-neovim-plugin-linux-x64.tar.gz -C "$install_dir"
```

The archive is already rooted like a Neovim plugin and includes the matching `versionlens-lsp` binary. When developing from this monorepo, add `packages/neovim-plugin` to `runtimepath` instead.

Without a bundled binary, the plugin downloads the matching `versionlens-lsp` release into `stdpath("data")/versionlens/<version>/` and verifies it against `SHA256SUMS` (set `auto_download = false` to disable). Pass server settings with `setup({ settings = { ... } })`. See `:help versionlens`.

## Setup

```lua
require("versionlens").setup({
  -- Optional: cmd = { "/path/to/versionlens-lsp" },
  keymaps = {
    refresh = nil,
    restart = nil,
  },
})
```

`setup()` is idempotent. Set `vim.g.versionlens_disable_auto_setup = 1` before plugin loading when your plugin manager should own initialization completely.

## Commands

- `:VersionLensStart`
- `:VersionLensStop`
- `:VersionLensRestart`
- `:VersionLensRefresh`
- `:VersionLensInfo`
- `:checkhealth versionlens`

## Development

```bash
make -C packages/neovim-plugin lint
make -C packages/neovim-plugin test
```

Tests use `plenary.nvim` through `tests/minimal_init.lua` and never load the user's Neovim configuration.

## License

[ISC](LICENSE)

## Attribution

VersionLens Redux is a fork of the original VersionLens extension by Peter Flannery and contributors. Neovim support uses the fork's shared Rust LSP server.
