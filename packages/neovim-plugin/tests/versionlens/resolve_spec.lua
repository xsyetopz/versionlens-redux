local download = require("versionlens.download")
local support = require("versionlens.support")
local version = require("versionlens.version")

local function touch_executable(path)
  vim.fn.mkdir(vim.fs.dirname(path), "p")
  vim.fn.writefile({ "#!/bin/sh" }, path)
  vim.uv.fs_chmod(path, tonumber("755", 8))
end

describe("server resolution", function()
  local root
  local original_env
  local original_data_dir
  local original_root
  local original_path
  local name = support.executable_name()

  before_each(function()
    root = vim.fn.tempname()
    vim.fn.mkdir(root, "p")
    original_env = vim.env.VERSIONLENS_LSP
    original_path = vim.env.PATH
    original_data_dir = download.data_dir
    original_root = support.plugin_root
    vim.env.VERSIONLENS_LSP = nil
    vim.env.PATH = "/nonexistent"
    download.data_dir = function()
      return vim.fs.joinpath(root, "data", "versionlens")
    end
    support.plugin_root = function()
      return vim.fs.joinpath(root, "plugin")
    end
  end)

  after_each(function()
    vim.env.VERSIONLENS_LSP = original_env
    vim.env.PATH = original_path
    download.data_dir = original_data_dir
    support.plugin_root = original_root
    vim.fn.delete(root, "rf")
  end)

  local function place_bundled()
    local target = support.editor_target()
    local path = vim.fs.joinpath(root, "plugin", "bin", target, name)
    touch_executable(path)
    return path
  end
  local function place_downloaded()
    local path = download.binary_path(version)
    touch_executable(path)
    return path
  end
  local function place_on_path()
    local directory = vim.fs.joinpath(root, "pathbin")
    touch_executable(vim.fs.joinpath(directory, name))
    vim.env.PATH = directory
    return vim.fs.joinpath(directory, name)
  end

  it("prefers cmd, then env, then bundled, then downloaded, then PATH", function()
    local env_path = vim.fs.joinpath(root, "envbin", name)
    touch_executable(env_path)
    local bundled = place_bundled()
    local downloaded = place_downloaded()
    place_on_path()
    vim.env.VERSIONLENS_LSP = env_path

    local cmd, source = support.resolve_cmd({ cmd = { "custom", "--flag" } })
    assert.are.same({ "custom", "--flag" }, cmd)
    assert.are.equal("cmd", source)

    cmd, source = support.resolve_cmd({})
    assert.are.same({ env_path }, cmd)
    assert.are.equal("env", source)

    vim.env.VERSIONLENS_LSP = nil
    cmd, source = support.resolve_cmd({})
    assert.are.same({ bundled }, cmd)
    assert.are.equal("bundled", source)

    vim.fn.delete(bundled)
    cmd, source = support.resolve_cmd({})
    assert.are.same({ downloaded }, cmd)
    assert.are.equal("downloaded", source)

    vim.fn.delete(downloaded)
    cmd, source = support.resolve_cmd({})
    assert.are.equal("path", source)
    assert.are.equal(vim.fn.resolve(vim.fn.exepath(name)), vim.fn.resolve(cmd[1]))
  end)

  it("returns nil when nothing is found", function()
    assert.is_nil(support.resolve_cmd({}))
  end)
end)

describe("download", function()
  local root
  local original = {}
  local calls
  local sums_body
  local hash_output

  before_each(function()
    root = vim.fn.tempname()
    vim.fn.mkdir(root, "p")
    original.data_dir = download.data_dir
    original.run = download.run
    original.sha256 = download.sha256
    original.triple = download.triple
    download.data_dir = function()
      return vim.fs.joinpath(root, "versionlens")
    end
    download.triple = function()
      return "x86_64-unknown-linux-gnu"
    end
    calls = {}
    local archive = download.archive_name(version, "x86_64-unknown-linux-gnu")
    sums_body = ("%s  %s\n"):format(("a"):rep(64), archive)
    hash_output = ("a"):rep(64)
    download.sha256 = function(_, callback)
      callback(hash_output)
    end
    download.run = function(argv, on_exit)
      table.insert(calls, argv)
      if argv[1] == "curl" then
        local out = argv[4]
        if out:match("SHA256SUMS$") then
          vim.fn.writefile({ (sums_body:gsub("\n$", "")) }, out)
        else
          vim.fn.writefile({ "archive" }, out)
        end
      elseif argv[1] == "tar" then
        touch_executable(vim.fs.joinpath(argv[5], "versionlens-lsp-x", support.executable_name()))
      end
      on_exit({ code = 0, stdout = "", stderr = "" })
    end
  end)

  after_each(function()
    download.data_dir = original.data_dir
    download.run = original.run
    download.sha256 = original.sha256
    download.triple = original.triple
    vim.fn.delete(root, "rf")
  end)

  it("names assets and URLs like the release workflow", function()
    assert.are.equal(
      "versionlens-lsp-1.2.3-x86_64-pc-windows-msvc.zip",
      download.archive_name("1.2.3", "x86_64-pc-windows-msvc")
    )
    assert.are.equal(
      "versionlens-lsp-1.2.3-aarch64-apple-darwin.tar.gz",
      download.archive_name("1.2.3", "aarch64-apple-darwin")
    )
    assert.are.equal(
      "https://github.com/xsyetopz/versionlens-redux/releases/download/v1.2.3/SHA256SUMS",
      download.release_url("1.2.3", "SHA256SUMS")
    )
  end)

  it("installs a verified binary and removes stale versions", function()
    vim.fn.mkdir(vim.fs.joinpath(root, "versionlens", "0.0.1"), "p")
    local done, err, path
    download.install(version, function(e, p)
      done, err, path = true, e, p
    end)
    assert.is_true(done)
    assert.is_nil(err)
    assert.are.equal(download.binary_path(version), path)
    assert.are.equal(1, vim.fn.executable(path))
    assert.are.equal(0, vim.fn.isdirectory(vim.fs.joinpath(root, "versionlens", "0.0.1")))
    assert.are.equal(
      0,
      vim.fn.isdirectory(vim.fs.joinpath(download.install_dir(version), ".download"))
    )
  end)

  it("deletes the download and names both sums on mismatch", function()
    hash_output = ("b"):rep(64)
    local err, path
    download.install(version, function(e, p)
      err, path = e, p
    end)
    assert.is_nil(path)
    assert.is_truthy(err:find(("a"):rep(64), 1, true))
    assert.is_truthy(err:find(("b"):rep(64), 1, true))
    assert.are.equal(0, vim.fn.isdirectory(download.install_dir(version)))
    assert.are.equal(0, vim.fn.filereadable(download.binary_path(version)))
  end)

  it("fails when SHA256SUMS has no entry", function()
    sums_body = ("%s  other.tar.gz\n"):format(("a"):rep(64))
    local err
    download.install(version, function(e)
      err = e
    end)
    assert.is_truthy(err:find("no entry", 1, true))
    assert.are.equal(0, vim.fn.isdirectory(download.install_dir(version)))
  end)

  it("parses SHA256SUMS entries", function()
    local sums = ("%s  a.tar.gz\n%s *b.zip\n"):format(("A"):rep(64), ("c"):rep(64))
    assert.are.equal(("a"):rep(64), download.expected_checksum(sums, "a.tar.gz"))
    assert.are.equal(("c"):rep(64), download.expected_checksum(sums, "b.zip"))
    assert.is_nil(download.expected_checksum(sums, "c.zip"))
  end)

  describe("start", function()
    local versionlens = require("versionlens")
    local original_start = vim.lsp.start
    local original_install = download.install
    local original_resolve = support.resolve_cmd
    local bufnr

    before_each(function()
      bufnr = vim.api.nvim_create_buf(true, false)
      vim.api.nvim_set_current_buf(bufnr)
      vim.api.nvim_buf_set_name(bufnr, ("/tmp/versionlens-test/%d/package.json"):format(bufnr))
      support.resolve_cmd = function()
        return nil
      end
    end)

    after_each(function()
      vim.lsp.start = original_start
      download.install = original_install
      support.resolve_cmd = original_resolve
      if vim.api.nvim_buf_is_valid(bufnr) then
        vim.api.nvim_buf_delete(bufnr, { force = true })
      end
    end)

    it("makes no network call when auto_download is false", function()
      versionlens.setup({ autostart = false, auto_download = false, notify = false })
      local installs = 0
      download.install = function()
        installs = installs + 1
      end
      assert.is_nil(versionlens.start(bufnr))
      assert.are.equal(0, installs)
      assert.are.equal(0, #calls)
    end)

    it("downloads when nothing is found and auto_download is on", function()
      versionlens.setup({ autostart = false, notify = false })
      local requested
      download.install = function(requested_version, callback)
        requested = requested_version
        callback("stubbed")
      end
      assert.is_nil(versionlens.start(bufnr))
      assert.are.equal(version, requested)
    end)
  end)
end)
