-- VersionLens for Neovim: starts versionlens-lsp for the manifests it reads and exposes its commands.
local M = {}

M.defaults = {
  -- Server command; nil looks at $VERSIONLENS_LSP, the bundled bin/, PATH, then the downloaded release.
  cmd = nil,
  auto_download = true,
  autostart = true,
  codelens = true,
  inlay_hints = true,
  -- The `versionlens` settings section, as in VS Code: { prereleases = true, timeoutMs = 5000, ... }.
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

--- Lua patterns for every manifest the server reads, asked from the server once.
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
  if M.config.on_attach then
    M.config.on_attach(client, bufnr)
  end
end

local handlers = {
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
  local settings =
    { versionlens = vim.tbl_isempty(M.config.settings) and vim.empty_dict() or M.config.settings }
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

local function execute(command, bufnr)
  local client = clients(bufnr)[1]
  if not client then
    notify("not running for this buffer", vim.log.levels.WARN)
    return
  end
  local arguments = command == "versionlens.clearCache" and {} or { vim.uri_from_bufnr(bufnr) }
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

--- Subcommands of :VersionLens.
M.actions = {
  refresh = function(bufnr)
    execute("versionlens.refresh", bufnr)
  end,
  updateAll = function(bufnr)
    execute("versionlens.updateAll", bufnr)
  end,
  toggle = function(bufnr)
    execute("versionlens.toggle", bufnr)
  end,
  clearCache = function(bufnr)
    execute("versionlens.clearCache", bufnr)
  end,
  restart = function(bufnr)
    for _, client in ipairs(clients(bufnr)) do
      client:stop()
    end
    patterns = nil
    vim.defer_fn(function()
      M.start(bufnr)
    end, 200)
  end,
  log = show_log,
  info = function()
    local cmd = M.cmd()
    notify(cmd and table.concat(cmd, " ") or "server not found")
  end,
}

--- Configures the plugin, registers :VersionLens, and starts on manifest buffers.
function M.setup(opts)
  M.config = vim.tbl_deep_extend("force", vim.deepcopy(M.defaults), opts or {})
  patterns = nil
  vim.api.nvim_create_user_command("VersionLens", function(args)
    local action = M.actions[args.args]
    if not action then
      notify("unknown subcommand " .. args.args, vim.log.levels.ERROR)
      return
    end
    action(vim.api.nvim_get_current_buf())
  end, {
    nargs = 1,
    force = true,
    complete = function()
      return vim.tbl_keys(M.actions)
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
