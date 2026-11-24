-- Drives the plugin against the live registries: nvim --headless --clean -l tests/live.lua <versionlens-lsp>
-- Every check goes through what a user reaches: :VersionLens, code actions, lenses, hover, setup().
vim.opt.runtimepath:prepend(vim.fn.getcwd())
local server = assert(arg[1], "usage: nvim --headless -l tests/live.lua <versionlens-lsp>")
local versionlens = require("versionlens")
local failures = 0
-- Registry and OSV lookups go over the network; 30 s covers a cold cache on a slow link.
local LIVE_MS = 30000

local function check(label, fn)
  local started = vim.uv.hrtime()
  local ok, err = pcall(fn)
  local ms = math.floor((vim.uv.hrtime() - started) / 1e6)
  print(
    ("%s %s (%d ms)%s"):format(
      ok and "ok  " or "FAIL",
      label,
      ms,
      ok and "" or ("\n     " .. tostring(err))
    )
  )
  failures = failures + (ok and 0 or 1)
end

local function wait(what, predicate)
  assert(vim.wait(LIVE_MS, predicate, 50), "timed out waiting for " .. what)
end

local root = vim.fn.tempname()
local function edit(name, lines)
  local path = vim.fs.joinpath(root, name)
  vim.fn.mkdir(root, "p")
  vim.fn.writefile(lines, path)
  vim.cmd.edit(path)
  return vim.api.nvim_get_current_buf()
end

local function client(bufnr)
  return vim.lsp.get_clients({ bufnr = bufnr, name = "versionlens" })[1]
end

local function hints(bufnr)
  return vim.lsp.inlay_hint.get({ bufnr = bufnr })
end

local function request(bufnr, method, params)
  local responses =
    assert(vim.lsp.buf_request_sync(bufnr, method, params, LIVE_MS), method .. " timed out")
  local response = select(2, next(responses))
  assert(response and not response.err, method .. ": " .. vim.inspect(response and response.err))
  return response.result
end

local function at_line(bufnr, line)
  return {
    textDocument = vim.lsp.util.make_text_document_params(bufnr),
    range = { start = { line = line, character = 0 }, ["end"] = { line = line, character = 0 } },
    position = { line = line, character = 2 },
    context = { diagnostics = {} },
  }
end

local function lines(bufnr)
  return vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)
end

versionlens.setup({ cmd = { server }, auto_download = false })
local old = { "requests==2.31.0", "flask>=2.0" }
local reqs = edit("requirements.txt", old)

check("requirements.txt attaches and gets an inlay hint per dependency", function()
  wait("the server to attach", function()
    return client(reqs) ~= nil
  end)
  wait("two inlay hints", function()
    return #hints(reqs) == 2
  end)
  local label = hints(reqs)[1].inlay_hint.label
  assert(type(label) == "string" and label:find("→"), "hint label " .. vim.inspect(label))
end)

check("requests 2.31.0 gets its OSV advisories, not a failed check", function()
  wait("an advisory diagnostic", function()
    for _, d in ipairs(vim.diagnostic.get(reqs)) do
      if d.lnum == 0 and d.source == "VersionLens" and d.code ~= "update-available" then
        return true
      end
    end
  end)
  for _, d in ipairs(vim.diagnostic.get(reqs)) do
    assert(d.code ~= "vulnerability-check-failed", d.message)
  end
end)

check("hover describes the dependency", function()
  local hover = request(reqs, "textDocument/hover", at_line(reqs, 0))
  local text = hover and hover.contents and (hover.contents.value or hover.contents)
  assert(type(text) == "string" and text:find("requests"), vim.inspect(hover))
end)

check("code lenses open the package page through window/showDocument", function()
  local lenses = request(reqs, "textDocument/codeLens", at_line(reqs, 0))
  local open
  for _, lens in ipairs(lenses or {}) do
    if lens.command and lens.command.command == "versionlens.open" then
      open = lens.command
    end
  end
  assert(open, "no versionlens.open lens in " .. vim.inspect(lenses))
  local opened
  local ui_open = vim.ui.open
  vim.ui.open = function(url)
    opened = url
    return nil, nil
  end
  client(reqs):exec_cmd(open, { bufnr = reqs })
  local ok = vim.wait(LIVE_MS, function()
    return opened ~= nil
  end, 50)
  vim.ui.open = ui_open
  assert(ok and opened:find("^https://"), "opened " .. vim.inspect(opened))
end)

check("the update code action rewrites the version", function()
  local actions = request(reqs, "textDocument/codeAction", at_line(reqs, 0))
  local update
  for _, action in ipairs(actions or {}) do
    if action.title:find("^Update requests") then
      update = update or action
    end
  end
  assert(update, "no update action in " .. vim.inspect(actions))
  if update.edit then
    vim.lsp.util.apply_workspace_edit(update.edit, client(reqs).offset_encoding)
  end
  if update.command then
    client(reqs):exec_cmd(update.command, { bufnr = reqs })
  end
  wait("requests to change", function()
    return lines(reqs)[1] ~= old[1]
  end)
end)

check(":VersionLens toggle hides and shows the hints", function()
  vim.cmd("VersionLens toggle")
  wait("the hints to go", function()
    return #hints(reqs) == 0
  end)
  vim.cmd("VersionLens toggle")
  wait("the hints to come back", function()
    return #hints(reqs) == 2
  end)
end)

check("setup() pushes new settings to the running server", function()
  versionlens.setup({ cmd = { server }, auto_download = false, settings = { inlayHints = false } })
  wait("the hints to go", function()
    return #hints(reqs) == 0
  end)
  versionlens.setup({ cmd = { server }, auto_download = false })
  wait("the hints to come back", function()
    return #hints(reqs) == 2
  end)
end)

check(":VersionLens clearCache then updateAll rewrites every dependency", function()
  vim.api.nvim_buf_set_lines(reqs, 0, -1, false, old)
  vim.cmd("VersionLens clearCache")
  vim.cmd("VersionLens updateAll")
  wait("both lines to change", function()
    local now = lines(reqs)
    return now[1] ~= old[1] and now[2] ~= old[2]
  end)
end)

check(":VersionLens refresh reaches the server log", function()
  local before = #versionlens.messages
  vim.cmd("VersionLens refresh")
  wait("the refresh log line", function()
    for i = before + 1, #versionlens.messages do
      if versionlens.messages[i]:find("command versionlens.refresh", 1, true) then
        return true
      end
    end
  end)
end)

check(":VersionLens log opens the server log", function()
  vim.cmd("VersionLens log")
  assert(
    vim.api.nvim_buf_get_name(0):find("versionlens://log", 1, true),
    vim.api.nvim_buf_get_name(0)
  )
  assert(#lines(0) > 0, "empty log")
  vim.cmd.close()
end)

check(":VersionLens info names the server", function()
  local said
  local notify = vim.notify
  vim.notify = function(message)
    said = message
  end
  vim.cmd("VersionLens info")
  vim.notify = notify
  assert(said and said:find(server, 1, true), vim.inspect(said))
end)

check(":VersionLens restart starts a new server", function()
  local before = client(reqs).id
  vim.cmd("VersionLens restart")
  wait("a new client", function()
    local now = client(reqs)
    return now ~= nil and now.id ~= before
  end)
  wait("hints from the new server", function()
    return #hints(reqs) == 2
  end)
end)

check("libraries.hxml attaches and gets hints", function()
  local hxml = edit("libraries.hxml", { "-lib heaps:1.10.0", "-lib format" })
  wait("the server to attach", function()
    return client(hxml) ~= nil
  end)
  wait("an inlay hint", function()
    return #hints(hxml) >= 1
  end)
end)

for _, c in ipairs(vim.lsp.get_clients()) do
  c:stop(true)
end
os.exit(failures == 0 and 0 or 1)
