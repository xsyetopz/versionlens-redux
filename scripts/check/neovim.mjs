#!/usr/bin/env bun
import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import process from "node:process";

const packageRoot = join("packages", "neovim-plugin");
const requiredPaths = [
  "README.md",
  "LICENSE",
  "doc/versionlens.txt",
  "lua/versionlens/config.lua",
  "lua/versionlens/health.lua",
  "lua/versionlens/init.lua",
  "lua/versionlens/support.lua",
  "lua/versionlens/version.lua",
  "plugin/versionlens.lua",
  "tests/minimal_init.lua",
  "tests/versionlens/setup_spec.lua",
  "tests/versionlens/support_spec.lua",
];

for (const relativePath of requiredPaths) {
  const path = join(packageRoot, relativePath);
  if (!existsSync(path)) {
    throw new Error(`Neovim package is missing ${path}`);
  }
}

const { version } = JSON.parse(readFileSync("package.json", "utf8"));
const versionModule = readFileSync(
  join(packageRoot, "lua", "versionlens", "version.lua"),
  "utf8",
);
if (versionModule.trim() !== `return "${version}"`) {
  throw new Error("Neovim package version is not synchronized");
}

const init = readFileSync(
  join(packageRoot, "lua", "versionlens", "init.lua"),
  "utf8",
);
for (const contract of [
  "vim.api.nvim_create_user_command",
  "vim.api.nvim_create_autocmd",
  "vim.keymap.set",
  "vim.lsp.start",
]) {
  if (!init.includes(contract)) {
    throw new Error(`Neovim adapter does not use required API: ${contract}`);
  }
}
if (init.includes("vim.cmd")) {
  throw new Error("Neovim adapter must use Lua APIs instead of vim.cmd");
}

const luaFiles = requiredPaths
  .filter((path) => path.endsWith(".lua"))
  .map((path) => join(packageRoot, path));
const luaCompiler = Bun.which("luac");
if (luaCompiler) {
  const syntax = Bun.spawnSync([luaCompiler, "-p", ...luaFiles]);
  if (syntax.exitCode !== 0) {
    process.stderr.write(syntax.stderr);
    process.exit(syntax.exitCode);
  }
}

const syntaxStatus = luaCompiler ? " and Lua syntax" : "";
console.log(`Verified Neovim package structure, version, APIs${syntaxStatus}.`);
