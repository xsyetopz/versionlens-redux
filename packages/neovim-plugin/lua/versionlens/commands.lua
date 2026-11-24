-- :VersionLens parsing, completion, server requests, and the buffer menu.
local M = {}

M.levels = { "latest", "major", "minor", "patch" }
M.toggles = { "show", "hide", "toggle" }

-- Canonical subcommand => { args = allowed values for its one argument }.
M.subcommands = {
  show = {},
  hide = {},
  toggle = {},
  prereleases = { args = M.toggles },
  update = { args = M.levels },
  workspace = { args = M.levels },
  sort = {},
  install = {},
  refresh = {},
  ["clear-cache"] = {},
  log = {},
  restart = {},
  info = {},
}

-- Older spellings and VS Code names.
M.aliases = {
  updateAll = "update",
  clearCache = "clear-cache",
  output = "log",
  showOutput = "log",
}

--- Subcommand names for completion, canonical and alias, sorted.
function M.names()
  local names = vim.tbl_keys(M.subcommands)
  vim.list_extend(names, vim.tbl_keys(M.aliases))
  table.sort(names)
  return names
end

--- Parses the text after :VersionLens into { name, arg }, or nil plus an error message.
function M.parse(text)
  local words = vim.split(vim.trim(text or ""), "%s+", { trimempty = true })
  local name = M.aliases[words[1] or ""] or words[1]
  local def = M.subcommands[name or ""]
  if not def then
    return nil, "unknown subcommand " .. tostring(words[1])
  end
  if #words > 2 or (#words == 2 and not def.args) then
    return nil, ("%s takes %s"):format(name, def.args and "one argument" or "no argument")
  end
  local arg = words[2]
  if arg and not vim.tbl_contains(def.args, arg) then
    return nil, ("%s: expected %s, got %s"):format(name, table.concat(def.args, "|"), arg)
  end
  if not arg then
    if name == "update" or name == "workspace" then
      arg = "latest"
    elseif name == "prereleases" then
      arg = "toggle"
    end
  end
  return { name = name, arg = arg }
end

--- Maps a parsed subcommand to a server command and its arguments; nil for client-only ones.
function M.request(spec, uri)
  local name, arg = spec.name, spec.arg
  if name == "show" or name == "hide" or name == "toggle" then
    return "versionlens." .. name, { uri }
  elseif name == "prereleases" then
    local verb = arg == "toggle" and "toggle" or arg
    return "versionlens." .. verb .. "Prereleases", { uri }
  elseif name == "update" then
    return "versionlens.updateAll", { uri, arg }
  elseif name == "workspace" then
    return "versionlens.updateWorkspace", { arg }
  elseif name == "sort" or name == "install" or name == "refresh" then
    return "versionlens." .. name, { uri }
  elseif name == "clear-cache" then
    return "versionlens.clearCache", {}
  end
  return nil
end

--- Completion candidates for a :VersionLens command line.
function M.complete(arglead, cmdline)
  local words = vim.split(cmdline, "%s+", { trimempty = true })
  local position = #words + (cmdline:match("%s$") and 1 or 0)
  local candidates = {}
  if position <= 2 then
    candidates = M.names()
  elseif position == 3 then
    local def = M.subcommands[M.aliases[words[2]] or words[2]]
    candidates = def and def.args or {}
  end
  return vim.tbl_filter(function(candidate)
    return vim.startswith(candidate, arglead)
  end, candidates)
end

--- Menu entries that apply to a buffer's status: { label, command } with command a :VersionLens line.
function M.menu(status)
  local entries = {}
  local function add(label, command)
    table.insert(entries, { label = label, command = command })
  end
  if status then
    add(status.enabled and "Hide versions" or "Show versions", status.enabled and "hide" or "show")
    add(status.prereleases and "Hide prereleases" or "Show prereleases", "prereleases toggle")
    if (status.outdated or 0) > 0 then
      for _, level in ipairs(M.levels) do
        add(("Update all (%s)"):format(level), "update " .. level)
      end
    end
    if status.sortable then
      add("Sort dependencies", "sort")
    end
    if type(status.install) == "string" then
      add("Install: " .. status.install, "install")
    end
    add("Refresh versions", "refresh")
    add("Update workspace (latest)", "workspace latest")
  end
  add("Clear cache", "clear-cache")
  add("Show log", "log")
  add("Restart server", "restart")
  add("Server info", "info")
  return entries
end

return M
