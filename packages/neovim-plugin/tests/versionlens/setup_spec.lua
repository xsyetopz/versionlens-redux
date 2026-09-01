local versionlens = require("versionlens")

describe("setup", function()
  before_each(function()
    local bufnr = vim.api.nvim_create_buf(true, false)
    vim.api.nvim_set_current_buf(bufnr)
    vim.api.nvim_buf_set_name(bufnr, "/tmp/versionlens-test/README.md")
  end)

  it("is idempotent and registers the public commands once", function()
    versionlens.setup({ autostart = false })
    versionlens.setup({ autostart = false })

    local commands = vim.api.nvim_get_commands({ builtin = false })
    assert.is_not_nil(commands.VersionLensStart)
    assert.is_not_nil(commands.VersionLensStop)
    assert.is_not_nil(commands.VersionLensRestart)
    assert.is_not_nil(commands.VersionLensRefresh)
    assert.is_not_nil(commands.VersionLensInfo)
  end)

  it("merges nested configuration with documented defaults", function()
    local resolved = versionlens.setup({
      autostart = false,
      codelens = { enabled = false },
      keymaps = { refresh = "<leader>vr" },
    })

    assert.is_false(resolved.codelens.enabled)
    assert.are.same({ "BufEnter", "InsertLeave" }, resolved.codelens.events)
    assert.are.equal("<leader>vr", resolved.keymaps.refresh)
    assert.is_true(resolved.notify)
  end)

  it("replaces previously configured keymaps", function()
    versionlens.setup({ autostart = false, keymaps = { refresh = "<leader>vr" } })
    assert.is_not_empty(vim.fn.maparg("<leader>vr", "n"))

    versionlens.setup({ autostart = false, keymaps = { restart = "<leader>vR" } })
    assert.is_empty(vim.fn.maparg("<leader>vr", "n"))
    assert.is_not_empty(vim.fn.maparg("<leader>vR", "n"))
  end)
end)
