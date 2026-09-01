vim.opt.runtimepath:prepend(vim.fn.getcwd())

local plenary = vim.env.PLENARY_DIR
if plenary and plenary ~= "" then
  vim.opt.runtimepath:prepend(plenary)
else
  vim.opt.runtimepath:prepend(vim.fs.joinpath(vim.fn.getcwd(), "..", "..", ".deps", "plenary.nvim"))
end

vim.g.versionlens_disable_auto_setup = 1
