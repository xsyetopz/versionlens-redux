local support = require("versionlens.support")

local M = {}

function M.check()
  vim.health.start("VersionLens Redux")
  local cmd = support.resolve_cmd(require("versionlens").config)
  if cmd then
    vim.health.ok("Found language server: " .. table.concat(cmd, " "))
  else
    vim.health.error(
      "versionlens-lsp was not found",
      "Install versionlens-lsp on PATH, set VERSIONLENS_LSP, or configure require('versionlens').setup({ cmd = ... })."
    )
  end
  if vim.fn.has("nvim-0.10") == 1 then
    vim.health.ok("Neovim 0.10 or newer")
  else
    vim.health.error("VersionLens Redux requires Neovim 0.10 or newer")
  end
end

return M
