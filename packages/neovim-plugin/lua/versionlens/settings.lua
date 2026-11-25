-- :VersionLens settings, Neovim's form of the VS Code settings page: a Lua buffer listing every
-- `versionlens` setting with its value and VS Code's description. Writing it applies the changes to
-- running servers and keeps what differs from setup() in a JSON file under stdpath("data"), so
-- setup() stays the base and the page's edits survive a restart, as VS Code's user settings do.
local M = {}

-- { key, type, default, description }, in VS Code's order. The server command is the plugin's
-- `cmd` option and the LSP traffic log is Neovim's own, so serverPath and trace.server are absent.
M.schema = {
  { "enabled", "boolean", true, "Show versions and vulnerabilities at all." },
  { "codeLens", "boolean", true, "Show update actions as code lenses above each dependency." },
  {
    "inlayHints",
    "boolean",
    true,
    "Show the newest version as an inlay hint after each dependency.",
  },
  { "prereleases", "boolean", false, "Offer prerelease versions as updates." },
  {
    "vulnerabilities",
    "boolean",
    true,
    "Check versions against OSV.dev and report known vulnerabilities.",
  },
  { "timeoutMs", "number", 10000, "How long one registry request may take, in milliseconds." },
  { "concurrency", "number", 32, "How many registry requests run at once." },
  { "cacheMinutes", "number", 60, "How long registry answers are reused, in minutes." },
  { "strictSsl", "boolean", true, "Reject registries whose TLS certificate does not verify." },
  {
    "githubToken",
    "string",
    "",
    "GitHub token for GitHub tag lookups; falls back to GITHUB_TOKEN, GH_TOKEN, then gh auth token.",
  },
  {
    "registries",
    "table",
    {},
    'URL template per registry id, such as { npm = "https://npm.example.com/{enc}" }.',
  },
  {
    "providers",
    "table",
    {},
    "Per registry id, overrides of cacheMinutes, strictSsl, and prereleaseTags, such as { npm = { cacheMinutes = 5 } }.",
  },
  {
    "authorization",
    "table",
    {},
    "Authorization header per registry URL prefix; :VersionLens auth keeps headers in its own file.",
  },
  {
    "install",
    "table",
    {},
    'Shell command per file type or registry id, such as { npm = "bun install" }.',
  },
  {
    "files",
    "table",
    {},
    'Extra manifests: glob to file type id, such as { ["deps/*.list"] = "requirements" }.',
  },
  {
    "indicators",
    "table",
    {},
    "Glyph shown before each version state in code lenses and inlay hints.",
  },
  {
    "prereleaseTags",
    "table",
    {},
    'Offer only prereleases with one of these tags, such as { "rc", "beta" }.',
  },
  {
    "updateSeverity",
    "string",
    "info",
    'Diagnostic severity of an outdated dependency: "off", "hint", "info", "warning", or "error".',
  },
  { "stats", "boolean", false, "Show a summary code lens above each manifest." },
  {
    "fileTypes",
    "table",
    {},
    'Read only these file type or registry ids, such as { "npm", "cargo" }; empty reads all.',
  },
  {
    "workspaceExclude",
    "table",
    {},
    'Globs the workspace update skips, such as { "examples/**" }.',
  },
}

local BUFFER = "versionlens://settings"

--- Where the page keeps the settings that differ from setup().
function M.path()
  return vim.fs.joinpath(vim.fn.stdpath("data"), "versionlens", "settings.json")
end

--- The kept settings; empty when there is no file. Errors name a corrupt file.
function M.load()
  local file = io.open(M.path(), "r")
  if not file then
    return {}
  end
  local text = file:read("*a")
  file:close()
  local ok, saved = pcall(vim.json.decode, text, { luanil = { object = true, array = true } })
  if not ok or type(saved) ~= "table" then
    error(M.path() .. " is not a JSON object")
  end
  return saved
end

--- Problems in a settings table: unknown keys and wrong value types, one string each.
function M.validate(settings)
  local kinds = {}
  for _, row in ipairs(M.schema) do
    kinds[row[1]] = row[2]
  end
  local problems = {}
  for key, value in pairs(settings) do
    if not kinds[key] then
      table.insert(problems, "unknown setting " .. key)
    elseif type(value) ~= kinds[key] then
      table.insert(problems, ("setting %s must be a %s"):format(key, kinds[key]))
    end
  end
  table.sort(problems)
  return problems
end

--- The page text for `settings`, every key shown with its value or its default.
function M.render(settings)
  local lines = {
    "-- VersionLens settings, the `versionlens` section of VS Code's settings page.",
    "-- :w applies them to running servers and keeps what differs from setup() in",
    "-- " .. M.path() .. ".",
    "-- The server command is setup({ cmd = ... }); the LSP traffic log is",
    '-- vim.lsp.log.set_level("trace"). :help versionlens-settings lists the values.',
    "return {",
  }
  for _, row in ipairs(M.schema) do
    local key, default, description = row[1], row[3], row[4]
    local value = settings[key]
    if value == nil then
      value = default
    end
    table.insert(lines, "  -- " .. description)
    local shown = vim.split(vim.inspect(value), "\n")
    shown[1] = key .. " = " .. shown[1]
    shown[#shown] = shown[#shown] .. ","
    for _, line in ipairs(shown) do
      table.insert(lines, "  " .. line)
    end
  end
  table.insert(lines, "}")
  return lines
end

--- The table the page text returns, or nil and the reason. The text runs with no globals.
function M.parse(text)
  local chunk, err = loadstring(text, "=" .. BUFFER)
  if not chunk then
    return nil, err
  end
  setfenv(chunk, {})
  local ok, value = pcall(chunk)
  if not ok then
    return nil, value
  end
  if type(value) ~= "table" then
    return nil, "the settings must be a table"
  end
  local problems = M.validate(value)
  if #problems > 0 then
    return nil, table.concat(problems, "; ")
  end
  return value
end

--- The settings of `page` that differ from `base` once defaults are filled in.
function M.changes(page, base)
  local changed = {}
  for _, row in ipairs(M.schema) do
    local key, default = row[1], row[3]
    local was = base[key]
    if was == nil then
      was = default
    end
    if page[key] ~= nil and not vim.deep_equal(page[key], was) then
      changed[key] = page[key]
    end
  end
  return changed
end

--- Keeps the settings of `page` that differ from `base` and returns the settings now in effect:
--- `base` with those changes on top.
function M.save(page, base)
  local changed = M.changes(page, base)
  require("versionlens.auth").write(M.path(), changed)
  return vim.tbl_extend("force", vim.deepcopy(base), changed)
end

-- The page buffer is made once; the latest open() decides what a write does.
local written

--- Opens the page showing `settings`. A write parses it and calls `on_write(page)`, which returns
--- the message to show or raises the reason the settings were not saved.
function M.open(settings, on_write)
  written = on_write
  local bufnr = vim.fn.bufnr(BUFFER)
  if bufnr == -1 then
    bufnr = vim.api.nvim_create_buf(true, false)
    vim.api.nvim_buf_set_name(bufnr, BUFFER)
    vim.bo[bufnr].buftype = "acwrite"
    vim.bo[bufnr].swapfile = false
    vim.api.nvim_create_autocmd("BufWriteCmd", {
      buffer = bufnr,
      callback = function()
        local text = table.concat(vim.api.nvim_buf_get_lines(bufnr, 0, -1, false), "\n")
        local page, err = M.parse(text)
        local ok, message = false, err
        if page then
          ok, message = pcall(written, page)
        end
        if not ok then
          vim.notify("VersionLens: settings not saved: " .. message, vim.log.levels.ERROR)
          return
        end
        vim.bo[bufnr].modified = false
        vim.notify("VersionLens: " .. message)
      end,
    })
  end
  vim.api.nvim_buf_set_lines(bufnr, 0, -1, false, M.render(settings))
  vim.bo[bufnr].modified = false
  vim.api.nvim_set_current_buf(bufnr)
  vim.bo[bufnr].filetype = "lua"
  return bufnr
end

return M
