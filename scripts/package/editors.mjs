#!/usr/bin/env bun

import process from "node:process";

const commands = [
  ["bun", "run", "package"],
  ["cargo", "build", "-p", "versionlens-lsp", "--release", "--locked"],
  [
    "cargo",
    "build",
    "--manifest-path",
    "packages/zed-extension/Cargo.toml",
    "--release",
    "--locked",
  ],
  ["bun", "scripts/package/zed.mjs"],
  ["gradle", "-p", "packages/jetbrains-plugin", "buildPlugin", "--no-daemon"],
  ["bun", "scripts/check/editors.mjs"],
];

for (const command of commands) {
  console.log(`$ ${command.join(" ")}`);
  const result = Bun.spawnSync(command, {
    stderr: "inherit",
    stdout: "inherit",
  });
  if (result.exitCode !== 0) {
    process.exit(result.exitCode);
  }
}
