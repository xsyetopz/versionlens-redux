local support = require("versionlens.support")

local M = {}

M.repository = "xsyetopz/versionlens-redux"

-- Generated from scripts/package/targets.json by scripts/generate/manifests.mjs.
local triples = {
  ["win32-x64"] = "x86_64-pc-windows-msvc",
  ["win32-arm64"] = "aarch64-pc-windows-msvc",
  ["linux-x64"] = "x86_64-unknown-linux-gnu",
  ["linux-arm64"] = "aarch64-unknown-linux-gnu",
  ["linux-armhf"] = "armv7-unknown-linux-gnueabihf",
  ["alpine-x64"] = "x86_64-unknown-linux-musl",
  ["alpine-arm64"] = "aarch64-unknown-linux-musl",
  ["darwin-x64"] = "x86_64-apple-darwin",
  ["darwin-arm64"] = "aarch64-apple-darwin",
}

local function is_musl()
  return vim.uv.fs_stat("/etc/alpine-release") ~= nil
end

-- Release triple for this machine, or nil on unsupported platforms.
function M.triple()
  local target = support.editor_target()
  if target and target:match("^linux%-") and is_musl() then
    target = target:gsub("^linux", "alpine")
  end
  return target and triples[target] or nil
end

function M.archive_name(version, triple)
  local extension = triple:find("windows", 1, true) and "zip" or "tar.gz"
  return ("versionlens-lsp-%s-%s.%s"):format(version, triple, extension)
end

function M.release_url(version, asset)
  return ("https://github.com/%s/releases/download/v%s/%s"):format(M.repository, version, asset)
end

-- Overridable so tests never touch the real data directory.
function M.data_dir()
  return vim.fs.joinpath(vim.fn.stdpath("data"), "versionlens")
end

function M.install_dir(version)
  return vim.fs.joinpath(M.data_dir(), version)
end

function M.binary_path(version)
  return vim.fs.joinpath(M.install_dir(version), support.executable_name())
end

-- Runs argv and calls on_exit({ code, stdout, stderr }) from the main loop.
-- Overridable so tests make no process or network calls.
function M.run(argv, on_exit)
  local ok, err = pcall(vim.system, argv, { text = true }, vim.schedule_wrap(on_exit))
  if not ok then
    vim.schedule(function()
      on_exit({ code = 127, stdout = "", stderr = tostring(err) })
    end)
  end
end

-- Expected hex digest for archive from sha256sum-style SHA256SUMS text.
function M.expected_checksum(sums, archive)
  for line in sums:gmatch("[^\r\n]+") do
    local hash, name = line:match("^%s*(%x+)%s+%*?(.-)%s*$")
    if hash and name == archive then
      return hash:lower()
    end
  end
  return nil
end

local function hash_command(path)
  if vim.fn.executable("sha256sum") == 1 then
    return { "sha256sum", path }, "first"
  elseif vim.fn.executable("shasum") == 1 then
    return { "shasum", "-a", "256", path }, "first"
  elseif vim.fn.executable("certutil") == 1 then
    return { "certutil", "-hashfile", path, "SHA256" }, "certutil"
  end
  return nil
end

function M.has_hash_tool()
  return hash_command("") ~= nil
end

-- bsdtar reads tar.gz and zip. On Windows use the system one, not whichever tar is on PATH.
function M.tar_command()
  local root = vim.env.SystemRoot
  if vim.fn.has("win32") == 1 and root and root ~= "" then
    return vim.fs.joinpath(root, "System32", "tar.exe")
  end
  return "tar"
end

function M.sha256(path, callback)
  local argv, style = hash_command(path)
  if not argv then
    return callback(nil, "no SHA-256 tool found (need sha256sum, shasum, or certutil)")
  end
  M.run(argv, function(result)
    if result.code ~= 0 then
      return callback(nil, "hashing failed: " .. (result.stderr or ""))
    end
    local hash
    if style == "certutil" then
      hash = ((result.stdout or ""):match("[^\r\n]+[\r\n]+([^\r\n]+)") or ""):gsub("%s", "")
    else
      hash = (result.stdout or ""):match("^%s*(%x+)")
    end
    if not hash or #hash ~= 64 then
      return callback(nil, "could not parse SHA-256 output")
    end
    callback(hash:lower())
  end)
end

local function remove_stale(version)
  local root = M.data_dir()
  for name, kind in vim.fs.dir(root) do
    if kind == "directory" and name ~= version then
      vim.fn.delete(vim.fs.joinpath(root, name), "rf")
    end
  end
end

-- Downloads, verifies, and installs the server for version. callback(err, path).
-- On any failure the install directory is removed, so no unverified binary remains.
function M.install(version, callback)
  local triple = M.triple()
  if not triple then
    local uname = vim.uv.os_uname()
    return callback(
      ("no versionlens-lsp release is available for %s %s"):format(uname.sysname, uname.machine)
    )
  end
  if vim.fn.executable("curl") ~= 1 then
    return callback("curl is required to download versionlens-lsp")
  end
  local directory = M.install_dir(version)
  local binary = M.binary_path(version)
  -- Per process, so concurrent Neovim instances never share or delete each other's files.
  local scratch = vim.fs.joinpath(directory, (".download-%d"):format(vim.uv.os_getpid()))
  local archive = M.archive_name(version, triple)
  local archive_path = vim.fs.joinpath(scratch, archive)
  local sums_path = vim.fs.joinpath(scratch, "SHA256SUMS")
  local extract_path = vim.fs.joinpath(scratch, "extract")

  local function fail(message)
    vim.fn.delete(scratch, "rf")
    vim.fn.delete(binary .. ".part")
    vim.uv.fs_rmdir(directory) -- only succeeds when empty
    callback(message)
  end

  local function finish()
    local found = vim.fs.find(support.executable_name(), { path = extract_path, limit = 1 })[1]
    if not found then
      return fail(archive .. " does not contain " .. support.executable_name())
    end
    local partial = binary .. ".part"
    if not vim.uv.fs_rename(found, partial) then
      return fail("could not move the server binary into place")
    end
    vim.uv.fs_chmod(partial, tonumber("755", 8))
    if not vim.uv.fs_rename(partial, binary) then
      return fail("could not install the server binary")
    end
    vim.fn.delete(scratch, "rf")
    remove_stale(version)
    callback(nil, binary)
  end

  local function verify()
    local sums = vim.fn.filereadable(sums_path) == 1
        and table.concat(vim.fn.readfile(sums_path), "\n")
      or ""
    local expected = M.expected_checksum(sums, archive)
    if not expected then
      return fail("SHA256SUMS has no entry for " .. archive)
    end
    M.sha256(archive_path, function(actual, err)
      if not actual then
        return fail(err)
      end
      if actual ~= expected then
        return fail(("%s checksum mismatch: expected %s, got %s"):format(archive, expected, actual))
      end
      vim.fn.mkdir(extract_path, "p")
      M.run({ M.tar_command(), "-xf", archive_path, "-C", extract_path }, function(result)
        if result.code ~= 0 then
          return fail("could not extract " .. archive .. ": " .. (result.stderr or ""))
        end
        finish()
      end)
    end)
  end

  vim.fn.mkdir(scratch, "p")
  local function fetch(asset, path, next_step)
    M.run({ "curl", "-fsSL", "-o", path, M.release_url(version, asset) }, function(result)
      if result.code ~= 0 then
        return fail(("could not download %s: %s"):format(asset, result.stderr or ""))
      end
      next_step()
    end)
  end
  fetch(archive, archive_path, function()
    fetch("SHA256SUMS", sums_path, verify)
  end)
end

return M
