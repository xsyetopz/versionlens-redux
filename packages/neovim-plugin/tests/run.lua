-- Headless checks against a real server: nvim --headless -l tests/run.lua <versionlens-lsp>
vim.opt.runtimepath:prepend(vim.fn.getcwd())
local server = assert(arg[1], "usage: nvim --headless -l tests/run.lua <versionlens-lsp>")
local versionlens = require("versionlens")
local failures = 0

local function check(label, fn)
  local ok, err = pcall(fn)
  print((ok and "ok   " or "FAIL ") .. label .. (ok and "" or ("\n     " .. tostring(err))))
  failures = failures + (ok and 0 or 1)
end

local function eq(actual, expected)
  assert(actual == expected, ("%s ~= %s"):format(vim.inspect(actual), vim.inspect(expected)))
end

local function edit(name, lines)
  local path = vim.fs.joinpath(vim.fn.tempname(), name)
  vim.fn.mkdir(vim.fs.dirname(path), "p")
  vim.fn.writefile(lines, path)
  vim.cmd.edit(path)
  return vim.api.nvim_get_current_buf()
end

versionlens.setup({ cmd = { server }, auto_download = false })

check("server globs become segment-anchored patterns", function()
  local patterns = versionlens.patterns({ server })
  assert(
    versionlens.supports("/p/requirements-dev.txt", patterns),
    "versionlens.supports('/p/requirements-dev.txt', patterns)"
  )
  assert(
    versionlens.supports("C:\\p\\Requirements\\base.txt", patterns),
    "versionlens.supports('C:\\p\\Requirements\\base.txt', patterns)"
  )
  assert(
    versionlens.supports("/p/libraries.hxml", patterns),
    "versionlens.supports('/p/libraries.hxml', patterns)"
  )
  for _, workflow in ipairs({ "/p/.github/workflows/ci.yml", "/p/.GitHub/workflows/a/b.yaml" }) do
    assert(versionlens.supports(workflow, patterns), "versionlens.supports('" .. workflow .. "')")
  end
  assert(versionlens.supports("/p/dockerfile", patterns), "lowercase dockerfile")
  assert(
    not versionlens.supports("/p/workflows/ci.yml", patterns),
    "not versionlens.supports('/p/workflows/ci.yml', patterns)"
  )
  assert(
    not versionlens.supports("/p/notes.txt", patterns),
    "not versionlens.supports('/p/notes.txt', patterns)"
  )
  assert(
    not versionlens.supports("/p/mypackage.json", patterns),
    "not versionlens.supports('/p/mypackage.json', patterns)"
  )
end)

check("a manifest attaches the server and answers code lenses", function()
  local bufnr = edit("package.json", { '{ "dependencies": { "left-pad": "^1.0.0" } }' })
  assert(
    vim.wait(5000, function()
      return #vim.lsp.get_clients({ bufnr = bufnr, name = "versionlens" }) == 1
    end),
    "server did not attach"
  )
  local responses = vim.lsp.buf_request_sync(bufnr, "textDocument/codeLens", {
    textDocument = vim.lsp.util.make_text_document_params(bufnr),
  }, 5000)
  local lenses = select(2, next(responses or {}))
  assert(lenses and lenses.result and #lenses.result > 0, "no code lenses")
  assert(
    vim.wait(5000, function()
      return #versionlens.messages > 0
    end),
    "server logged nothing"
  )
end)

check("other files do not attach", function()
  local bufnr = edit("notes.txt", { "hello" })
  vim.wait(300)
  assert(
    #vim.lsp.get_clients({ bufnr = bufnr, name = "versionlens" }) == 0,
    "#vim.lsp.get_clients({ bufnr = bufnr, name = 'versionlens' }) == 0"
  )
end)

check(":VersionLens parses every subcommand", function()
  local commands = require("versionlens.commands")
  local function request(text, uri)
    local spec, err = commands.parse(text)
    assert(spec, text .. ": " .. tostring(err))
    local command, arguments = commands.request(spec, uri)
    return command, arguments
  end
  local cases = {
    { "show", "versionlens.show", { "u" } },
    { "hide", "versionlens.hide", { "u" } },
    { "toggle", "versionlens.toggle", { "u" } },
    { "prereleases", "versionlens.togglePrereleases", { "u" } },
    { "prereleases show", "versionlens.showPrereleases", { "u" } },
    { "prereleases hide", "versionlens.hidePrereleases", { "u" } },
    { "prereleases toggle", "versionlens.togglePrereleases", { "u" } },
    { "update", "versionlens.updateAll", { "u", "latest" } },
    { "update patch", "versionlens.updateAll", { "u", "patch" } },
    { "updateAll minor", "versionlens.updateAll", { "u", "minor" } },
    { "workspace", "versionlens.updateWorkspace", { "latest" } },
    { "workspace major", "versionlens.updateWorkspace", { "major" } },
    { "sort", "versionlens.sort", { "u" } },
    { "install", "versionlens.install", { "u" } },
    { "refresh", "versionlens.refresh", { "u" } },
    { "clear-cache", "versionlens.clearCache", {} },
    { "clearCache", "versionlens.clearCache", {} },
  }
  for _, case in ipairs(cases) do
    local command, arguments = request(case[1], "u")
    assert(command == case[2], case[1] .. " -> " .. tostring(command))
    assert(vim.deep_equal(arguments, case[3]), case[1] .. " arguments " .. vim.inspect(arguments))
  end
  for _, local_only in ipairs({ "log", "output", "restart", "info" }) do
    local spec = assert(commands.parse(local_only))
    assert(commands.request(spec, "u") == nil, local_only .. " stays in the client")
  end
  assert(commands.parse("output").name == "log", "output aliases log")
  for _, bad in ipairs({ "", "bogus", "update huge", "show now", "update major extra" }) do
    local spec, err = commands.parse(bad)
    assert(spec == nil and err, "should reject " .. vim.inspect(bad))
  end
end)

check(":VersionLens completes subcommands and arguments", function()
  local commands = require("versionlens.commands")
  local first = commands.complete("", "VersionLens ")
  for _, name in ipairs(vim.tbl_keys(commands.subcommands)) do
    assert(vim.tbl_contains(first, name), "missing " .. name)
  end
  assert(vim.deep_equal(commands.complete("pre", "VersionLens pre"), { "prereleases" }))
  assert(vim.deep_equal(commands.complete("", "VersionLens update "), commands.levels))
  assert(vim.deep_equal(commands.complete("ma", "VersionLens workspace ma"), { "major" }))
  assert(vim.deep_equal(commands.complete("s", "VersionLens prereleases s"), { "show" }))
  assert(#commands.complete("", "VersionLens sort ") == 0, "sort takes no argument")
  assert(
    vim.deep_equal(vim.fn.getcompletion("VersionLens upd", "cmdline"), { "update", "updateAll" }),
    vim.inspect(vim.fn.getcompletion("VersionLens upd", "cmdline"))
  )
end)

check("status renders glyphs and counts, Unicode and ASCII", function()
  local status = require("versionlens.status")
  local base = {
    enabled = true,
    prereleases = false,
    busy = false,
    dependencies = 5,
    outdated = 0,
    failed = 0,
    vulnerable = 0,
  }
  local function render(changes, opts)
    return status.render(vim.tbl_extend("force", base, changes), opts)
  end
  assert(status.render(nil) == "", "no status renders nothing")
  assert(render({}) == "◉ ✓", render({}))
  eq(render({ outdated = 3, failed = 1, vulnerable = 2 }), "◉ ↑3 ✗1 ⚠2")
  eq(render({ enabled = false, outdated = 1 }), "◌ ↑1")
  eq(render({ prereleases = true }), "◉ ✓ β")
  eq(render({ busy = true }, { frame = 1 }), "◉ ⠋")
  eq(render({ busy = true }, { frame = 2 }), "◉ ⠙")
  assert(render({ busy = true }, { frame = 11 }) == "◉ ⠋", "spinner wraps")
  assert(render({ dependencies = 0 }) == "◉", "no dependencies, no check mark")
  local ascii = { ascii = true }
  eq(render({ outdated = 3, failed = 1, vulnerable = 2 }, ascii), "VL ^3 x1 !2")
  eq(render({ enabled = false, prereleases = true, outdated = 1 }, ascii), "vl- ^1 pre")
  eq(render({ busy = true }, { ascii = true, frame = 2 }), "VL /")
  assert(render({ outdated = 1 }, { glyphs = { outdated = "U" } }) == "◉ U1", "glyph override")
end)

check("versionlens/status is stored per buffer and shown by statusline()", function()
  local status = require("versionlens.status")
  status.reset()
  local bufnr = edit("package.json", { "{}" })
  local uri = vim.uri_from_bufnr(bufnr)
  assert(status.get(bufnr) == nil, "no status before the server reports")
  status.set({
    uri = uri,
    enabled = true,
    prereleases = true,
    busy = false,
    dependencies = 4,
    outdated = 2,
    failed = 0,
    vulnerable = 1,
    sortable = true,
    install = "npm install",
  })
  assert(status.get(bufnr).outdated == 2, "stored by uri")
  assert(versionlens.statusline(bufnr) == "◉ ↑2 ⚠1 β", versionlens.statusline(bufnr))
  assert(versionlens.statusline() == "◉ ↑2 ⚠1 β", "default is the current buffer")
  versionlens.config.ascii = true
  assert(versionlens.statusline(bufnr) == "VL ^2 !1 pre", versionlens.statusline(bufnr))
  versionlens.config.ascii = false
  assert(status.set({ enabled = true }) == nil, "payload without a uri is ignored")
  status.reset()
  assert(versionlens.statusline(bufnr) == "", "reset clears")
end)

check("the menu lists what applies to the buffer", function()
  local commands = require("versionlens.commands")
  local function labels(status)
    return vim.tbl_map(function(entry)
      return entry.label
    end, commands.menu(status))
  end
  local function has(list, label)
    return vim.tbl_contains(list, label)
  end
  local none = labels(nil)
  assert(
    not has(none, "Refresh versions") and has(none, "Clear cache") and has(none, "Show log"),
    "menu without a status"
  )
  local idle = labels({ enabled = true, prereleases = false, outdated = 0, sortable = false })
  assert(has(idle, "Hide versions") and has(idle, "Show prereleases"), "idle menu toggles")
  assert(
    not has(idle, "Sort dependencies") and not has(idle, "Update all (major)"),
    "idle menu omits what does not apply"
  )
  assert(not vim.iter(idle):any(function(label)
    return label:find("^Install")
  end), "no install entry without an install command")
  local full = commands.menu({
    enabled = false,
    prereleases = true,
    outdated = 2,
    sortable = true,
    install = "bun install",
  })
  local by_label = {}
  for _, entry in ipairs(full) do
    by_label[entry.label] = entry.command
  end
  eq(by_label["Show versions"], "show")
  eq(by_label["Hide prereleases"], "prereleases toggle")
  eq(by_label["Update all (patch)"], "update patch")
  eq(by_label["Sort dependencies"], "sort")
  eq(by_label["Install: bun install"], "install")
  eq(by_label["Update workspace (latest)"], "workspace latest")
  for _, entry in ipairs(full) do
    assert(commands.parse(entry.command), "menu command does not parse: " .. entry.command)
  end
end)

check("settings validation names unknown keys and wrong types", function()
  assert(
    #versionlens.validate_settings({ timeoutMs = 10, registries = {}, strictSsl = true }) == 0,
    "valid settings pass"
  )
  local problems = versionlens.validate_settings({ timeoutMs = "x", nope = 1 })
  assert(
    vim.deep_equal(problems, { "setting timeoutMs must be a number", "unknown setting nope" }),
    vim.inspect(problems)
  )
end)

check("SHA256SUMS lookup", function()
  local install = require("versionlens.install")
  assert(install.checksum("ABC  a.tar.gz\ndef *b.zip\n", "b.zip") == "def", "listed archive")
  assert(
    install.checksum("abc  a.tar.gz\n", "c.zip") == nil,
    "install.checksum('abc  a.tar.gz\n', 'c.zip') == nil"
  )
end)

for _, client in ipairs(vim.lsp.get_clients()) do
  client:stop(true)
end
os.exit(failures == 0 and 0 or 1)
