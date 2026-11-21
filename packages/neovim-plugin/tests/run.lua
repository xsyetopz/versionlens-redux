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

check(":VersionLens has every subcommand", function()
  for _, sub in ipairs({ "refresh", "updateAll", "toggle", "clearCache", "restart", "log", "info" }) do
    assert(versionlens.actions[sub], sub)
  end
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
