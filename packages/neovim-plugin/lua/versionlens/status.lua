-- Per-buffer `versionlens/status` notifications and their statusline/winbar rendering.
local M = {}

M.glyphs = {
  unicode = {
    shown = "◉",
    hidden = "◌",
    ok = "✓",
    outdated = "↑",
    failed = "✗",
    vulnerable = "⚠",
    prereleases = "β",
    spinner = { "⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏" },
  },
  ascii = {
    shown = "VL",
    hidden = "vl-",
    ok = "ok",
    outdated = "^",
    failed = "x",
    vulnerable = "!",
    prereleases = "pre",
    spinner = { "|", "/", "-", "\\" },
  },
}

-- Lens and hint glyphs sent as the server's `indicators`: the server's defaults are the VS Code
-- emoji, which terminals draw at uneven widths, so Neovim sends text glyphs in their place.
M.indicators = {
  unicode = {
    latest = "✓",
    update = "↑",
    checking = "⋯",
    failed = "✗",
    unreadable = "?",
    vulnerable = "⚠",
    newer = "→",
    satisfies = "◐",
    noMatch = "○",
  },
  ascii = {
    latest = "ok",
    update = "^",
    checking = "...",
    failed = "x",
    unreadable = "?",
    vulnerable = "!",
    newer = "->",
    satisfies = "~",
    noMatch = "-",
  },
}

-- Status tables by normalized file path: the server names buffers by URI.
local by_path = {}
local frame = 1
local timer = nil

local function path_of(uri)
  local ok, fname = pcall(vim.uri_to_fname, uri)
  return vim.fs.normalize(ok and fname or uri)
end

--- The latest status for a buffer (default: the current one), or nil before the server reports.
function M.get(bufnr)
  if bufnr == nil or bufnr == 0 then
    bufnr = vim.api.nvim_get_current_buf()
  end
  return by_path[vim.fs.normalize(vim.api.nvim_buf_get_name(bufnr))]
end

local function any_busy()
  for _, status in pairs(by_path) do
    if status.busy then
      return true
    end
  end
  return false
end

local function redraw()
  pcall(vim.cmd.redrawstatus, { bang = true })
end

local function sync_timer()
  if any_busy() and not timer then
    timer = vim.uv.new_timer()
    timer:start(
      100,
      100,
      vim.schedule_wrap(function()
        frame = frame + 1
        redraw()
      end)
    )
  elseif timer and not any_busy() then
    timer:stop()
    timer:close()
    timer = nil
  end
end

--- Stores a `versionlens/status` payload; returns it, or nil when malformed.
function M.set(params)
  if type(params) ~= "table" or type(params.uri) ~= "string" then
    return nil
  end
  by_path[path_of(params.uri)] = params
  sync_timer()
  redraw()
  return params
end

--- Forgets a buffer's status, for example when its server stops.
function M.clear(bufnr)
  by_path[vim.fs.normalize(vim.api.nvim_buf_get_name(bufnr))] = nil
  sync_timer()
end

--- Forgets every status.
function M.reset()
  by_path = {}
  sync_timer()
end

--- Renders a status into glyphs and counts; "" without a status.
--- opts: ascii (boolean), frame (spinner index), glyphs (overrides by name).
function M.render(status, opts)
  if not status then
    return ""
  end
  opts = opts or {}
  local g =
    vim.tbl_extend("force", opts.ascii and M.glyphs.ascii or M.glyphs.unicode, opts.glyphs or {})
  local parts = { status.enabled and g.shown or g.hidden }
  if status.busy then
    table.insert(parts, g.spinner[((opts.frame or 1) - 1) % #g.spinner + 1])
  end
  local outdated = status.outdated or 0
  local failed = status.failed or 0
  local vulnerable = status.vulnerable or 0
  if outdated > 0 then
    table.insert(parts, g.outdated .. outdated)
  end
  if failed > 0 then
    table.insert(parts, g.failed .. failed)
  end
  if vulnerable > 0 then
    table.insert(parts, g.vulnerable .. vulnerable)
  end
  if not status.busy and (status.dependencies or 0) > 0 and outdated + failed + vulnerable == 0 then
    table.insert(parts, g.ok)
  end
  if status.prereleases then
    table.insert(parts, g.prereleases)
  end
  return table.concat(parts, " ")
end

--- Statusline text for a buffer (default: the current one); config holds ascii and glyphs.
function M.statusline(bufnr, config)
  config = config or {}
  return M.render(M.get(bufnr), { ascii = config.ascii, glyphs = config.glyphs, frame = frame })
end

return M
