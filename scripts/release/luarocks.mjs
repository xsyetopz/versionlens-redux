#!/usr/bin/env bun
import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";

import { parseSemver } from "./semver.mjs";

const [, , version] = Bun.argv;
if (!version) {
  throw new Error(
    "usage: bun scripts/release/luarocks.mjs <major.minor.patch> [output-directory]",
  );
}
parseSemver(version);

const outputDirectory = Bun.argv[3] ?? "dist";
const output = join(outputDirectory, `versionlens-redux-${version}-1.rockspec`);
const rockspec = `rockspec_format = "3.0"
package = "versionlens-redux"
version = "${version}-1"

description = {
  summary = "VersionLens dependency diagnostics and code lenses for Neovim",
  detailed = [[VersionLens Redux starts the shared versionlens-lsp server and renders dependency diagnostics and code lenses in supported manifests.]],
  homepage = "https://github.com/xsyetopz/versionlens-redux",
  license = "ISC",
  labels = { "neovim", "lsp", "dependencies" },
}

dependencies = {
  "lua >= 5.1",
}

source = {
  url = "https://github.com/xsyetopz/versionlens-redux/archive/refs/tags/v${version}.tar.gz",
  dir = "versionlens-redux-${version}/packages/neovim-plugin",
}

build = {
  type = "builtin",
  modules = {
    ["versionlens"] = "lua/versionlens/init.lua",
    ["versionlens.config"] = "lua/versionlens/config.lua",
    ["versionlens.health"] = "lua/versionlens/health.lua",
    ["versionlens.support"] = "lua/versionlens/support.lua",
    ["versionlens.version"] = "lua/versionlens/version.lua",
  },
  copy_directories = { "doc", "plugin" },
}
`;

mkdirSync(outputDirectory, { recursive: true });
writeFileSync(output, rockspec);
console.log(output);
