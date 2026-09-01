#!/usr/bin/env bun
import { chmodSync, copyFileSync, mkdirSync, rmSync } from "node:fs";
import { join } from "node:path";

import { resolveNativeEditorTarget } from "./editor-targets.mjs";

const { editorTarget, executableName, platform, sourceParts } =
  resolveNativeEditorTarget(Bun.argv.slice(2), "Neovim");
const packageRoot = join("packages", "neovim-plugin");
const binaryDirectory = join(packageRoot, "bin", editorTarget);
const bundledBinary = join(binaryDirectory, executableName);
const output = join(
  "dist",
  `versionlens-redux-neovim-plugin-${editorTarget}.tar.gz`,
);

mkdirSync(binaryDirectory, { recursive: true });
mkdirSync("dist", { recursive: true });
try {
  copyFileSync(join(...sourceParts), bundledBinary);
  if (platform !== "win32") {
    chmodSync(bundledBinary, 0o755);
  }
  const result = Bun.spawnSync([
    "tar",
    "-czf",
    output,
    "-C",
    packageRoot,
    "README.md",
    "LICENSE",
    "doc",
    "lua",
    "plugin",
    "bin",
  ]);
  if (result.exitCode !== 0) {
    throw new Error(
      `Could not create ${output}: ${new TextDecoder().decode(result.stderr)}`,
    );
  }
} finally {
  rmSync(join(packageRoot, "bin"), { force: true, recursive: true });
}

console.log(`Packaged ${output}`);
