#!/usr/bin/env bun

import process from "node:process";

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

export { run };
