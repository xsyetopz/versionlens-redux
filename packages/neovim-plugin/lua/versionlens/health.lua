local support = require("versionlens.support")

local M = {}

function M.check()
  vim.health.start("VersionLens Redux")
  local config = require("versionlens").config
  local cmd, source = support.resolve_cmd(config)
  if cmd then
    vim.health.ok(("Found language server (%s): %s"):format(source, table.concat(cmd, " ")))
  elseif config.auto_download then
    local download = require("versionlens.download")
    vim.health.warn("versionlens-lsp was not found", "It will be downloaded on first start.")
    if vim.fn.executable("curl") == 1 then
      vim.health.ok("curl found")
    else
      vim.health.error("curl is required for auto_download")
    end
    local tar = download.tar_command()
    if vim.fn.executable(tar) == 1 then
      vim.health.ok("tar found: " .. tar)
    else
      vim.health.error("tar is required for auto_download")
    end
    if download.has_hash_tool() then
      vim.health.ok("SHA-256 tool found")
    else
      vim.health.error("auto_download needs sha256sum, shasum, or certutil")
    end
    if not download.triple() then
      vim.health.error("No versionlens-lsp release exists for this platform")
    end
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
