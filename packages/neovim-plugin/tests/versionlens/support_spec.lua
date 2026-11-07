local support = require("versionlens.support")

describe("manifest support", function()
  it("recognizes exact manifest names and Dockerfile variants", function()
    assert.is_true(support.supports("/workspace/package.json"))
    assert.is_true(support.supports("/workspace/.nvmrc"))
    assert.is_true(support.supports("/workspace/.node-version"))
    assert.is_true(support.supports("/workspace/.bun-version"))
    assert.is_true(support.supports("/workspace/rust-toolchain"))
    assert.is_true(support.supports("/workspace/rust-toolchain.toml"))
    assert.is_true(support.supports("/workspace/Dockerfile.dev"))
    assert.is_true(support.supports("C:\\workspace\\Cargo.toml"))
  end)

  it("recognizes supported extensions case-insensitively", function()
    assert.is_true(support.supports("/workspace/Directory.Build.PROPS"))
    assert.is_true(support.supports("/workspace/dependencies.TOML"))
  end)

  it("rejects unrelated files", function()
    assert.is_false(support.supports("/workspace/README.md"))
    assert.is_false(support.supports("/workspace/Dockerfilex"))
    assert.is_false(support.supports(""))
  end)
end)

describe("root directory", function()
  local project
  local original_cwd

  before_each(function()
    original_cwd = vim.uv.cwd()
    local temporary = vim.fn.tempname()
    vim.fn.mkdir(vim.fs.joinpath(temporary, ".git"), "p")
    project = vim.fs.normalize(vim.uv.fs_realpath(temporary))
    vim.fn.mkdir(vim.fs.joinpath(project, ".claude", "worktrees", "agent", "launcher"), "p")
    vim.fn.writefile(
      { "gitdir: elsewhere" },
      vim.fs.joinpath(project, ".claude", "worktrees", "agent", ".git")
    )
    vim.fn.mkdir(vim.fs.joinpath(project .. "-other", "app"), "p")
    vim.fn.writefile({ "{}" }, vim.fs.joinpath(project .. "-other", "app", "package.json"))
  end)

  after_each(function()
    vim.cmd("silent! %bwipeout!")
    vim.cmd.cd(original_cwd)
    vim.fn.delete(project, "rf")
    vim.fn.delete(project .. "-other", "rf")
  end)

  local function root_of(path)
    vim.cmd.edit(path)
    return support.root_dir(0, require("versionlens.config").resolve())
  end

  it("keeps the project Neovim started in as the root of nested checkouts", function()
    vim.cmd.cd(project)
    local nested =
      vim.fs.joinpath(project, ".claude", "worktrees", "agent", "launcher", "package.json")
    assert.are.equal(project, vim.fs.normalize(vim.uv.fs_realpath(root_of(nested))))
  end)

  it("uses the nearest marker for files outside that project", function()
    vim.cmd.cd(project)
    local outside = vim.fs.joinpath(project .. "-other", "app", "package.json")
    assert.are.equal(
      vim.fs.joinpath(project .. "-other", "app"),
      vim.fs.normalize(vim.uv.fs_realpath(root_of(outside)))
    )
  end)

  it("uses the nearest marker when Neovim did not start in a project", function()
    vim.cmd.cd(vim.fs.joinpath(project, ".claude"))
    local nested =
      vim.fs.joinpath(project, ".claude", "worktrees", "agent", "launcher", "package.json")
    assert.are.equal(
      vim.fs.joinpath(project, ".claude", "worktrees", "agent"),
      vim.fs.normalize(vim.uv.fs_realpath(root_of(nested)))
    )
  end)
end)
