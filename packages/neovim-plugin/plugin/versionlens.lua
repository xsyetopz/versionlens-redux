if vim.g.loaded_versionlens_redux == 1 then
  return
end
vim.g.loaded_versionlens_redux = 1

if vim.g.versionlens_disable_auto_setup ~= 1 then
  require("versionlens").setup()
end
