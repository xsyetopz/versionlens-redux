-- Downloads the versionlens-lsp release that matches this plugin, verified against SHA256SUMS.
local M = {}

M.repository = "xsyetopz/versionlens-redux"

local TRIPLES = {
  ["windows-x86_64"] = "x86_64-pc-windows-msvc",
  ["windows-aarch64"] = "aarch64-pc-windows-msvc",
  ["linux-x86_64"] = "x86_64-unknown-linux-gnu",
  ["linux-aarch64"] = "aarch64-unknown-linux-gnu",
  ["linux-armv7l"] = "armv7-unknown-linux-gnueabihf",
  ["darwin-x86_64"] = "x86_64-apple-darwin",
  ["darwin-arm64"] = "aarch64-apple-darwin",
}

local waiting = nil

local function windows()
  return vim.fn.has("win32") == 1
end

--- The release target triple for this machine, or nil.
function M.triple()
  local uname = vim.uv.os_uname()
  local system = windows() and "windows" or uname.sysname:lower()
  local machine = ({ amd64 = "x86_64", AMD64 = "x86_64", arm64 = "aarch64", ARM64 = "aarch64" })[uname.machine]
    or uname.machine
  if system == "darwin" and machine == "aarch64" then
    machine = "arm64"
  end
  local triple = TRIPLES[system .. "-" .. machine]
  if triple and system == "linux" and vim.uv.fs_stat("/etc/alpine-release") then
    triple = triple:gsub("gnu$", "musl")
  end
  return triple
end

--- Where the downloaded server lives.
function M.binary()
  local file = windows() and "versionlens-lsp.exe" or "versionlens-lsp"
  return vim.fs.joinpath(
    vim.fn.stdpath("data"),
    "versionlens",
    require("versionlens.version"),
    file
  )
end

--- The SHA-256 that SHA256SUMS lists for archive, or nil.
function M.checksum(sums, archive)
  for line in sums:gmatch("[^\r\n]+") do
    local hash, file = line:match("^%s*(%x+)%s+%*?(.-)%s*$")
    if file == archive then
      return hash:lower()
    end
  end
end

local function read(path)
  local file = io.open(path, "rb")
  if not file then
    return ""
  end
  local data = file:read("*a")
  file:close()
  return data
end

local function run(argv, on_ok, fail)
  vim.system(
    argv,
    { text = true },
    vim.schedule_wrap(function(result)
      if result.code ~= 0 then
        return fail(("%s failed: %s"):format(argv[1], result.stderr or ""))
      end
      on_ok()
    end)
  )
end

--- Installs the server once, then calls each queued callback; report receives progress and errors.
function M.ensure(callback, report)
  if waiting then
    table.insert(waiting, callback)
    return
  end
  waiting = { callback }
  local version = require("versionlens.version")
  local triple = M.triple()
  local function finish(err)
    local queued = waiting
    waiting = nil
    if err then
      return report(
        "cannot install versionlens-lsp " .. version .. ": " .. err,
        vim.log.levels.ERROR
      )
    end
    report("installed versionlens-lsp " .. version)
    for _, queued_callback in ipairs(queued) do
      queued_callback()
    end
  end
  if not triple then
    return finish(
      "no release for " .. vim.uv.os_uname().sysname .. " " .. vim.uv.os_uname().machine
    )
  end
  if vim.fn.executable("curl") ~= 1 then
    return finish("curl is required")
  end
  local binary = M.binary()
  local directory = vim.fs.dirname(binary)
  local scratch = vim.fs.joinpath(directory, ".download-" .. vim.uv.os_getpid())
  local archive = ("versionlens-lsp-%s-%s.%s"):format(
    version,
    triple,
    windows() and "zip" or "tar.gz"
  )
  local base = ("https://github.com/%s/releases/download/v%s/"):format(M.repository, version)
  local function fail(message)
    vim.fn.delete(scratch, "rf")
    finish(message)
  end
  vim.fn.mkdir(scratch, "p")
  report("downloading versionlens-lsp " .. version)
  local archive_path = vim.fs.joinpath(scratch, archive)
  local sums_path = vim.fs.joinpath(scratch, "SHA256SUMS")
  run({ "curl", "-fsSL", "-o", archive_path, base .. archive }, function()
    run({ "curl", "-fsSL", "-o", sums_path, base .. "SHA256SUMS" }, function()
      local expected = M.checksum(read(sums_path), archive)
      local actual = vim.fn.sha256(read(archive_path))
      if expected ~= actual then
        return fail(
          ("checksum of %s is %s, SHA256SUMS lists %s"):format(archive, actual, tostring(expected))
        )
      end
      run({ "tar", "-xf", archive_path, "-C", scratch }, function()
        local found = vim.fs.find(vim.fs.basename(binary), { path = scratch, limit = 1 })[1]
        if not found or not vim.uv.fs_rename(found, binary) then
          return fail(archive .. " has no " .. vim.fs.basename(binary))
        end
        vim.uv.fs_chmod(binary, tonumber("755", 8))
        vim.fn.delete(scratch, "rf")
        finish()
      end, fail)
    end, fail)
  end, fail)
end

return M
