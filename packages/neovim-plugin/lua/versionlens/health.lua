local support = require("versionlens.support")

local M = {}

function M.check()
  vim.health.start("VersionLens Redux")
  local config = require("versionlens").config
  local cmd, source = support.resolve_cmd(config)
  if cmd then
    vim.health.ok(("Found language server (%s): %s"):format(source, table.concat(cmd, " ")))
  elseif config.auto_download then
    vim.health.warn(
      "versionlens-lsp was not found",
      "It will be downloaded on first start. Requires curl, tar, and sha256sum, shasum, or certutil."
    )
  else
    vim.health.error("versionlens-lsp was not found", {
      "Install versionlens-lsp on PATH, set VERSIONLENS_LSP, or configure setup({ cmd = ... }).",
      "Or enable auto_download.",
    })
  end
  if vim.fn.has("nvim-0.10") == 1 then
    vim.health.ok("Neovim 0.10 or newer")
  else
    vim.health.error("VersionLens Redux requires Neovim 0.10 or newer")
  end
end

return M
