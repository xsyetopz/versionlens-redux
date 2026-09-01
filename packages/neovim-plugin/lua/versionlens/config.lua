local M = {}

M.defaults = {
  autostart = true,
  cmd = nil,
  codelens = {
    enabled = true,
    events = { "BufEnter", "InsertLeave" },
  },
  keymaps = {
    refresh = nil,
    restart = nil,
  },
  notify = true,
  on_attach = nil,
  root_dir = nil,
  root_markers = {
    ".git",
    "Cargo.toml",
    "package.json",
    "pnpm-workspace.yaml",
    "pyproject.toml",
    "go.mod",
  },
}

function M.resolve(opts)
  return vim.tbl_deep_extend("force", {}, M.defaults, opts or {})
end

return M
