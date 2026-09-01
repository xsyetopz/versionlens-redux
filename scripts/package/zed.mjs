#!/usr/bin/env bun
import { chmodSync, copyFileSync, mkdirSync, rmSync } from "node:fs";
import { join } from "node:path";
import process from "node:process";

import { resolveNativeEditorTarget } from "./editor-targets.mjs";

const { editorTarget, executableName, platform, sourceParts } =
  resolveNativeEditorTarget(Bun.argv.slice(2), "Zed");
const source = join(...sourceParts);
const packageRoot = join("packages", "zed-extension");
const bundledBinary = join(packageRoot, "bin", executableName);
const output = join(
  "dist",
  `versionlens-redux-zed-extension-${editorTarget}.tar.gz`,
);

mkdirSync(join(packageRoot, "bin"), { recursive: true });
mkdirSync("dist", { recursive: true });
copyFileSync(source, bundledBinary);
if (platform !== "win32") {
  const executablePermissions = 0o755;
  chmodSync(bundledBinary, executablePermissions);
}

const result = Bun.spawnSync([
  "tar",
  "-czf",
  output,
  "-C",
  packageRoot,
  "Cargo.toml",
  "Cargo.lock",
  "extension.toml",
  "LICENSE",
  "README.md",
  "src",
  "bin",
]);
rmSync(join(packageRoot, "bin"), { force: true, recursive: true });
if (result.exitCode !== 0) {
  process.stderr.write(result.stderr);
  process.exit(result.exitCode);
}
console.log(`Packaged ${output}`);
