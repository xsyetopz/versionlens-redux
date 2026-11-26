vim.opt.runtimepath:prepend(vim.fn.getcwd())
vim.opt.runtimepath:prepend(
  vim.env.PLENARY_DIR or vim.fs.joinpath(vim.fn.getcwd(), "..", "..", ".deps", "plenary.nvim")
)
vim.g.versionlens_disable_auto_setup = 1
