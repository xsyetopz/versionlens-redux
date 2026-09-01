local config = require("versionlens.config")
local support = require("versionlens.support")

local M = {
  config = config.resolve(),
}

local group_name = "VersionLensRedux"
local client_name = "versionlens"
local active_keymaps = {}

local function notify(message, level)
  if M.config.notify then
    vim.notify(message, level or vim.log.levels.INFO, { title = "VersionLens Redux" })
  end
end

function M.refresh(bufnr)
  if not M.config.codelens.enabled then
    return
  end
  bufnr = bufnr or vim.api.nvim_get_current_buf()
  vim.lsp.codelens.refresh({ bufnr = bufnr })
end

function M.start(bufnr)
  bufnr = bufnr or vim.api.nvim_get_current_buf()
  local path = vim.api.nvim_buf_get_name(bufnr)
  if not support.supports(path) then
    return nil
  end
  local cmd = support.resolve_cmd(M.config)
  if not cmd then
    notify(
      "Could not find versionlens-lsp. Install it on PATH, set VERSIONLENS_LSP, or configure cmd.",
      vim.log.levels.ERROR
    )
    return nil
  end

  local user_on_attach = M.config.on_attach
  return vim.lsp.start({
    name = client_name,
    cmd = cmd,
    root_dir = support.root_dir(bufnr, M.config),
    on_attach = function(client, attached_bufnr)
      M.refresh(attached_bufnr)
      if user_on_attach then
        user_on_attach(client, attached_bufnr)
      end
    end,
  }, {
    bufnr = bufnr,
    reuse_client = function(client, candidate)
      return client.name == candidate.name and client.config.root_dir == candidate.root_dir
    end,
  })
end

function M.stop(bufnr)
  bufnr = bufnr or vim.api.nvim_get_current_buf()
  for _, client in ipairs(vim.lsp.get_clients({ bufnr = bufnr, name = client_name })) do
    client:stop()
  end
end

function M.restart(bufnr)
  bufnr = bufnr or vim.api.nvim_get_current_buf()
  M.stop(bufnr)
  vim.schedule(function()
    M.start(bufnr)
  end)
end

function M.info()
  local cmd = support.resolve_cmd(M.config)
  if cmd then
    notify("Language server: " .. table.concat(cmd, " "))
  else
    notify("Language server not found", vim.log.levels.WARN)
  end
end

local function register_commands()
  vim.api.nvim_create_user_command("VersionLensStart", function()
    M.start()
  end, { desc = "Start VersionLens Redux for the current buffer", force = true })
  vim.api.nvim_create_user_command("VersionLensStop", function()
    M.stop()
  end, { desc = "Stop VersionLens Redux for the current buffer", force = true })
  vim.api.nvim_create_user_command("VersionLensRestart", function()
    M.restart()
  end, { desc = "Restart VersionLens Redux for the current buffer", force = true })
  vim.api.nvim_create_user_command("VersionLensRefresh", function()
    M.refresh()
  end, { desc = "Refresh VersionLens Redux code lenses", force = true })
  vim.api.nvim_create_user_command("VersionLensInfo", function()
    M.info()
  end, { desc = "Show VersionLens Redux runtime information", force = true })
end

local function register_keymaps()
  for _, lhs in ipairs(active_keymaps) do
    pcall(vim.keymap.del, "n", lhs)
  end
  active_keymaps = {}
  if M.config.keymaps.refresh then
    local lhs = M.config.keymaps.refresh
    vim.keymap.set("n", lhs, function()
      M.refresh()
    end, { desc = "Refresh VersionLens" })
    table.insert(active_keymaps, lhs)
  end
  if M.config.keymaps.restart then
    local lhs = M.config.keymaps.restart
    vim.keymap.set("n", lhs, function()
      M.restart()
    end, { desc = "Restart VersionLens" })
    table.insert(active_keymaps, lhs)
  end
end

local function register_autocommands()
  local group = vim.api.nvim_create_augroup(group_name, { clear = true })
  if M.config.autostart then
    vim.api.nvim_create_autocmd({ "BufReadPost", "BufNewFile" }, {
      group = group,
      callback = function(event)
        M.start(event.buf)
      end,
    })
  end
  if M.config.codelens.enabled then
    vim.api.nvim_create_autocmd(M.config.codelens.events, {
      group = group,
      callback = function(event)
        M.refresh(event.buf)
      end,
    })
  end
end

function M.setup(opts)
  M.config = config.resolve(opts)
  register_commands()
  register_keymaps()
  register_autocommands()
  if M.config.autostart then
    local bufnr = vim.api.nvim_get_current_buf()
    if vim.api.nvim_buf_is_loaded(bufnr) then
      M.start(bufnr)
    end
  end
  return M.config
end

return M
