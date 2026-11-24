-- VersionLens for Neovim: starts versionlens-lsp for the manifests it reads and exposes its commands.
local M = {}

M.defaults = {
  -- Server command; nil looks at $VERSIONLENS_LSP, the bundled bin/, PATH, then the downloaded release.
  cmd = nil,
  auto_download = true,
  autostart = true,
  codelens = true,
  inlay_hints = true,
  -- Statusline glyphs: ASCII instead of Unicode, and per-glyph overrides (see versionlens.status).
  ascii = false,
  glyphs = {},
  -- Set each window's winbar to the status component when it is empty.
  winbar = false,
  -- The `versionlens` settings section, as in VS Code: { prereleases = true, timeoutMs = 5000, ... }.
  -- Keys: see :help versionlens-settings.
  settings = {},
  on_attach = nil,
}

M.config = vim.deepcopy(M.defaults)
M.messages = {}

local name = "versionlens"
local patterns = nil
local MAX_MESSAGES = 2000

local function notify(message, level)
  vim.notify("VersionLens: " .. message, level or vim.log.levels.INFO)
end

local function record(line)
  table.insert(M.messages, os.date("%H:%M:%S ") .. line)
  if #M.messages > MAX_MESSAGES then
    table.remove(M.messages, 1)
  end
end

--- The server command, or nil with the reason none was found.
function M.cmd()
  if M.config.cmd then
    return M.config.cmd
  end
  local env = vim.env.VERSIONLENS_LSP
  if env and env ~= "" then
    return { env }
  end
  local file = vim.fn.has("win32") == 1 and "versionlens-lsp.exe" or "versionlens-lsp"
  local bundled = vim.api.nvim_get_runtime_file("bin/" .. file, false)[1]
  if bundled then
    return { bundled }
  end
  if vim.fn.executable("versionlens-lsp") == 1 then
    return { "versionlens-lsp" }
  end
  local installed = require("versionlens.install").binary()
  if vim.fn.executable(installed) == 1 then
    return { installed }
  end
  return nil
end

-- A server glob such as `requirements/*.txt` becomes a Lua pattern anchored at a path segment.
local function to_pattern(glob)
  local escaped = glob:lower():gsub("[%^%$%(%)%%%.%[%]%+%-%?]", "%%%0"):gsub("%*", "[^/]*")
  return "/" .. escaped .. "$"
end

--- Lua patterns for every manifest the server reads, asked from the server once, plus the
--- globs of the `files` setting.
function M.patterns(cmd)
  if patterns then
    return patterns
  end
  local result =
    vim.system(vim.list_extend(vim.deepcopy(cmd), { "--file-types" }), { text = true }):wait(10000)
  local ok, decoded = pcall(vim.json.decode, result.stdout or "")
  if result.code ~= 0 or not ok or type(decoded.globs) ~= "table" then
    error(("%s --file-types failed: %s"):format(cmd[1], result.stderr or result.stdout or ""))
  end
  patterns = vim.tbl_map(to_pattern, decoded.globs)
  local files = M.config and M.config.settings and M.config.settings.files or {}
  for glob in pairs(files) do
    table.insert(patterns, to_pattern(glob))
  end
  return patterns
end

--- Whether the server reads the file at path.
function M.supports(path, list)
  local normalized = "/" .. path:gsub("\\", "/"):lower()
  for _, pattern in ipairs(list) do
    if normalized:find(pattern) then
      return true
    end
  end
  return false
end

local function on_attach(client, bufnr)
  if M.config.codelens then
    if vim.lsp.codelens.enable then
      vim.lsp.codelens.enable(true, { bufnr = bufnr })
    else
      vim.lsp.codelens.refresh({ bufnr = bufnr })
      vim.api.nvim_create_autocmd({ "BufEnter", "InsertLeave" }, {
        buffer = bufnr,
        callback = function()
          vim.lsp.codelens.refresh({ bufnr = bufnr })
        end,
      })
    end
  end
  if M.config.inlay_hints then
    vim.lsp.inlay_hint.enable(true, { bufnr = bufnr })
  end
  if M.config.winbar then
    for _, win in ipairs(vim.fn.win_findbuf(bufnr)) do
      if vim.wo[win].winbar == "" then
        vim.wo[win].winbar = "%{%v:lua.require'versionlens'.statusline()%}"
      end
    end
  end
  if M.config.on_attach then
    M.config.on_attach(client, bufnr)
  end
end

local handlers = {
  ["versionlens/status"] = function(_, params)
    require("versionlens.status").set(params)
  end,
  ["window/logMessage"] = function(_, params)
    record(params.message)
    return vim.NIL
  end,
}

-- Neovim 0.12 refreshes enabled code lenses itself; older versions need the handler.
if not vim.lsp.codelens.enable then
  handlers["workspace/codeLens/refresh"] = function(_, _, ctx)
    local client = vim.lsp.get_client_by_id(ctx.client_id)
    for bufnr in pairs(client and client.attached_buffers or {}) do
      vim.lsp.codelens.refresh({ bufnr = bufnr })
    end
    return vim.NIL
  end
end

-- The server reads the `versionlens` section; an empty table must encode as a JSON object.
local function server_settings()
  return {
    versionlens = vim.tbl_isempty(M.config.settings) and vim.empty_dict() or M.config.settings,
  }
end

--- Starts or reuses the server for bufnr when it is a supported manifest.
function M.start(bufnr)
  bufnr = bufnr or vim.api.nvim_get_current_buf()
  local path = vim.api.nvim_buf_get_name(bufnr)
  if path == "" or vim.bo[bufnr].buftype ~= "" then
    return
  end
  local cmd = M.cmd()
  if not cmd then
    if M.config.auto_download then
      require("versionlens.install").ensure(function()
        M.start(bufnr)
      end, notify)
    else
      notify("versionlens-lsp not found; set cmd, $VERSIONLENS_LSP, or PATH", vim.log.levels.ERROR)
    end
    return
  end
  local ok, list = pcall(M.patterns, cmd)
  if not ok then
    notify(list, vim.log.levels.ERROR)
    return
  end
  if not M.supports(path, list) then
    return
  end
  local settings = server_settings()
  return vim.lsp.start({
    name = name,
    cmd = vim.list_extend(vim.deepcopy(cmd), { "--stdio" }),
    root_dir = vim.fs.root(bufnr, { ".git" }) or vim.fs.dirname(path),
    settings = settings,
    init_options = settings,
    handlers = handlers,
    on_attach = on_attach,
  }, { bufnr = bufnr })
end

local function clients(bufnr)
  return vim.lsp.get_clients({ bufnr = bufnr, name = name })
end

local function execute(command, arguments, bufnr)
  local client = clients(bufnr)[1] or vim.lsp.get_clients({ name = name })[1]
  if not client then
    notify("not running; open a manifest first", vim.log.levels.WARN)
    return
  end
  client:request(
    "workspace/executeCommand",
    { command = command, arguments = arguments },
    function(err)
      if err then
        notify(err.message, vim.log.levels.ERROR)
      end
    end,
    bufnr
  )
end

local function show_log()
  vim.cmd("botright new")
  local buf = vim.api.nvim_get_current_buf()
  vim.bo[buf].buftype = "nofile"
  vim.bo[buf].bufhidden = "wipe"
  vim.api.nvim_buf_set_name(buf, "versionlens://log")
  vim.api.nvim_buf_set_lines(buf, 0, -1, false, M.messages)
end

local function restart(bufnr)
  for _, client in ipairs(clients(bufnr)) do
    client:stop()
  end
  patterns = nil
  require("versionlens.status").reset()
  vim.defer_fn(function()
    M.start(bufnr)
  end, 200)
end

local function info(bufnr)
  local cmd = M.cmd()
  local status = require("versionlens.status").get(bufnr)
  local lines = { cmd and table.concat(cmd, " ") or "server not found" }
  if status then
    table.insert(
      lines,
      ("dependencies %d, outdated %d, failed %d, vulnerable %d, shown %s, prereleases %s"):format(
        status.dependencies or 0,
        status.outdated or 0,
        status.failed or 0,
        status.vulnerable or 0,
        tostring(status.enabled),
        tostring(status.prereleases)
      )
    )
  end
  notify(table.concat(lines, "\n"))
end

--- Runs the :VersionLens line `text` for bufnr; returns false when it does not parse.
function M.run(text, bufnr)
  bufnr = bufnr or vim.api.nvim_get_current_buf()
  local commands = require("versionlens.commands")
  local spec, err = commands.parse(text)
  if not spec then
    notify(err, vim.log.levels.ERROR)
    return false
  end
  if spec.name == "log" then
    show_log()
  elseif spec.name == "restart" then
    restart(bufnr)
  elseif spec.name == "info" then
    info(bufnr)
  else
    local command, arguments = commands.request(spec, vim.uri_from_bufnr(bufnr))
    execute(command, arguments, bufnr)
  end
  return true
end

--- Picks a command that applies to the buffer's status, like the VS Code overflow menu.
function M.menu(bufnr)
  bufnr = bufnr or vim.api.nvim_get_current_buf()
  local entries = require("versionlens.commands").menu(require("versionlens.status").get(bufnr))
  vim.ui.select(entries, {
    prompt = "VersionLens",
    format_item = function(entry)
      return entry.label
    end,
  }, function(entry)
    if entry then
      M.run(entry.command, bufnr)
    end
  end)
end

--- Statusline or winbar text: glyphs and counts for the current buffer.
function M.statusline(bufnr)
  return require("versionlens.status").statusline(bufnr, M.config)
end

--- Problems in a settings table: unknown keys and wrong value types, one string each.
function M.validate_settings(settings)
  local kinds = {
    enabled = "boolean",
    codeLens = "boolean",
    inlayHints = "boolean",
    prereleases = "boolean",
    vulnerabilities = "boolean",
    strictSsl = "boolean",
    timeoutMs = "number",
    concurrency = "number",
    cacheMinutes = "number",
    githubToken = "string",
    registries = "table",
    authorization = "table",
    install = "table",
    files = "table",
    indicators = "table",
    prereleaseTags = "table",
    updateSeverity = "string",
    stats = "boolean",
    fileTypes = "table",
    workspaceExclude = "table",
  }
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

--- Configures the plugin, registers :VersionLens, and starts on manifest buffers.
--- Running servers receive the new settings at once.
function M.setup(opts)
  M.config = vim.tbl_deep_extend("force", vim.deepcopy(M.defaults), opts or {})
  for _, problem in ipairs(M.validate_settings(M.config.settings)) do
    notify(problem, vim.log.levels.WARN)
  end
  patterns = nil
  for _, client in ipairs(vim.lsp.get_clients({ name = name })) do
    -- vim.lsp.start reuses a running client, so its settings would otherwise stay the old ones.
    client.settings = server_settings()
    client:notify("workspace/didChangeConfiguration", { settings = client.settings })
  end
  vim.api.nvim_create_user_command("VersionLens", function(args)
    if args.args == "" then
      M.menu()
    else
      M.run(args.args)
    end
  end, {
    nargs = "*",
    force = true,
    desc = "VersionLens commands; no argument opens a menu",
    complete = function(arglead, cmdline)
      return require("versionlens.commands").complete(arglead, cmdline)
    end,
  })
  local group = vim.api.nvim_create_augroup("VersionLens", { clear = true })
  if M.config.autostart then
    vim.api.nvim_create_autocmd({ "BufReadPost", "BufNewFile" }, {
      group = group,
      callback = function(event)
        M.start(event.buf)
      end,
    })
    for _, bufnr in ipairs(vim.api.nvim_list_bufs()) do
      if vim.api.nvim_buf_is_loaded(bufnr) then
        M.start(bufnr)
      end
    end
  end
end

return M
