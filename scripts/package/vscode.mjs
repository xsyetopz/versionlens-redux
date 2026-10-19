#!/usr/bin/env bun
import { readFileSync } from "node:fs";
import { join } from "node:path";
import process from "node:process";

import { rustTargetFor, stageLspBinary } from "./stage-lsp.mjs";

function run(command, options = {}) {
  const result = Bun.spawnSync(command, {
    ...options,
    stdout: "inherit",
    stderr: "inherit",
  });
  if (result.exitCode !== 0) {
    process.exit(result.exitCode);
  }
}

const [, , vscodeTarget, archivesDir] = Bun.argv;
if (!(vscodeTarget && archivesDir)) {
  throw new Error(
    "usage: bun scripts/package/vscode.mjs <vscode-target> <build-lsp-archives-dir>",
  );
}
const serverDirectory = join("packages", "vscode-extension", "server");
const server = stageLspBinary(
  archivesDir,
  rustTargetFor(vscodeTarget),
  serverDirectory,
);
run(["bun", "scripts/package/extension.mjs"]);
const { version } = JSON.parse(readFileSync("package.json", "utf8"));
const output = `versionlens-redux-${version}-${vscodeTarget}.vsix`;
run(
  [
    "bunx",
    "vsce",
    "package",
    "--no-dependencies",
    "--target",
    vscodeTarget,
    "--out",
    output,
  ],
  { cwd: "packages/vscode-extension" },
);
run([
  "bun",
  "scripts/package/vsix.mjs",
  `packages/vscode-extension/${output}`,
  server,
  vscodeTarget,
]);
console.log(`Packaged packages/vscode-extension/${output}`);
