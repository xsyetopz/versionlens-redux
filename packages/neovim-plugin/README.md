# VersionLens Redux for Neovim

The Neovim plugin starts the shared Rust `versionlens-lsp` server and renders dependency diagnostics and code lenses in supported manifests.

## Requirements

- Neovim 0.10 or newer.
- A server binary (see Server binary below).
- `curl`, `tar`, and one of `sha256sum`, `shasum`, or `certutil` for the automatic download.

## Installation

Download the release archive matching your platform and architecture, then extract it as a normal Neovim package. For example:

```bash
install_dir="${XDG_DATA_HOME:-$HOME/.local/share}/nvim/site/pack/versionlens/start/versionlens-redux"
mkdir -p "$install_dir"
tar -xzf versionlens-redux-neovim-plugin-linux-x64.tar.gz -C "$install_dir"
```

The archive is already rooted like a Neovim plugin and includes the matching `versionlens-lsp` binary. When developing from this monorepo, add `packages/neovim-plugin` to `runtimepath` instead.

See `:help versionlens`.

## Server binary

The plugin looks for `versionlens-lsp` in this order:

1. The `cmd` option.
2. The `VERSIONLENS_LSP` environment variable.
3. A bundled `bin/<target>/versionlens-lsp` in the plugin directory. Here `<target>` is an editor target such as `darwin-arm64`; only the download archive name uses a Rust triple.
4. A repository build at `target/debug/versionlens-lsp` (monorepo checkout).
5. A previous download in `stdpath("data")/versionlens/<version>/`.
6. `versionlens-lsp` on `PATH`.

If none is found and `auto_download` is true (default), the plugin downloads `versionlens-lsp-<version>-<rust-triple>.tar.gz` (`.zip` on Windows, for example `versionlens-lsp-<version>-x86_64-apple-darwin.tar.gz`) from the GitHub release with `curl`, verifies it against the release `SHA256SUMS`, extracts it to `stdpath("data")/versionlens/<version>/`, and removes older versions. On a checksum or download failure nothing is installed.

Manual install: download the `versionlens-lsp-<version>-<rust-triple>` archive from the [releases page](https://github.com/xsyetopz/versionlens-redux/releases), extract the binary, and point `cmd` or `VERSIONLENS_LSP` at it (or put it on `PATH`). Set `auto_download = false` to disable the download.

## Setup

```lua
require("versionlens").setup({
  -- Optional: cmd = { "/path/to/versionlens-lsp" },
  keymaps = {
    refresh = nil,
    restart = nil,
  },
  -- Answered to the server's workspace/configuration request for "versionlens".
  settings = {
    diagnostics = { updateSeverity = "warn" },
  },
})
```

`settings` uses the same keys as the VS Code `versionlens` settings (see `contributes.configuration` in `../vscode-extension/package.json`). `diagnostics.updateSeverity` sets the severity of update-available diagnostics: `error`, `warn`, `info` (default), `hint`, or `off`. `off` hides them; vulnerability diagnostics are unaffected.

`setup()` is idempotent. Set `vim.g.versionlens_disable_auto_setup = 1` before plugin loading when your plugin manager should own initialization completely.

## Private registries

Neovim does not implement the `versionlens/authorization` request, so the server uses its other credential sources:

- For `api.github.com`: `GITHUB_TOKEN`, then `GH_TOKEN`, then the output of `gh auth token`, which is only tried after GitHub answers 401, 403, or 429.
- Registry configuration files such as `.npmrc`, `.yarnrc.yml`, and `NuGet.config`.

Start Neovim with the variables set. Tokens are never written to logs, diagnostics, or the cache.

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
