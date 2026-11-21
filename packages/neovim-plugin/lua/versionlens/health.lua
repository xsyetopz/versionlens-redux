-- :checkhealth versionlens
local M = {}

function M.check()
  local versionlens = require("versionlens")
  vim.health.start("VersionLens")
  if vim.fn.has("nvim-0.11") == 1 then
    vim.health.ok("Neovim " .. tostring(vim.version()))
  else
    vim.health.error("Neovim 0.11 or newer is required")
  end
  local cmd = versionlens.cmd()
  if not cmd then
    local hint = versionlens.config.auto_download and "it downloads when a manifest opens"
      or "set cmd, $VERSIONLENS_LSP, or put versionlens-lsp on PATH"
    vim.health.warn("versionlens-lsp not found; " .. hint)
    return
  end
  local result =
    vim.system(vim.list_extend(vim.deepcopy(cmd), { "--version" }), { text = true }):wait(5000)
  if result.code ~= 0 then
    vim.health.error(("%s --version failed: %s"):format(cmd[1], result.stderr or ""))
    return
  end
  vim.health.ok(vim.trim(result.stdout) .. " (" .. table.concat(cmd, " ") .. ")")
  local ok, patterns = pcall(versionlens.patterns, cmd)
  if ok then
    vim.health.ok(#patterns .. " manifest patterns")
  else
    vim.health.error(patterns)
  end
end

return M
