#!/usr/bin/env bun
import {
  chmodSync,
  copyFileSync,
  mkdirSync,
  readFileSync,
  rmSync,
} from "node:fs";
import { join } from "node:path";

import {
  arch as hostArchitecture,
  platform as hostPlatform,
} from "node:process";

const nativeEditorTargets = new Map(
  JSON.parse(
    readFileSync(new URL("./targets.json", import.meta.url), "utf8"),
  ).targets.map(({ vscode, rust }) => [vscode, rust]),
);

function resolveNativeEditorTarget(args, packageName) {
  const [requestedPlatform, requestedArchitecture, requestedRustTarget] = args;
  const requestedTarget = args.length > 0;
  if (requestedTarget && args.length !== 3) {
    throw new Error(
      `${packageName} target packaging requires platform, architecture, and Rust target arguments.`,
    );
  }
  if (
    requestedTarget &&
    (requestedPlatform !== hostPlatform ||
      requestedArchitecture !== hostArchitecture)
  ) {
    throw new Error(
      `Cannot package ${requestedPlatform}-${requestedArchitecture} on ${hostPlatform}-${hostArchitecture}; use a native runner.`,
    );
  }

  const platform = requestedPlatform ?? hostPlatform;
  const architecture = requestedArchitecture ?? hostArchitecture;
  const editorTarget = `${platform}-${architecture}`;
  const expectedRustTarget = nativeEditorTargets.get(editorTarget);
  if (!expectedRustTarget) {
    throw new Error(
      `Unsupported ${packageName} package target: ${editorTarget}`,
    );
  }
  if (requestedRustTarget && requestedRustTarget !== expectedRustTarget) {
    throw new Error(
      `${editorTarget} requires Rust target ${expectedRustTarget}, received ${requestedRustTarget}.`,
    );
  }

  const executableName =
    platform === "win32" ? "versionlens-lsp.exe" : "versionlens-lsp";
  const sourceParts = ["target"];
  if (requestedRustTarget) {
    sourceParts.push(requestedRustTarget);
  }
  sourceParts.push("release", executableName);

  return {
    architecture,
    editorTarget,
    executableName,
    platform,
    rustTarget: requestedRustTarget,
    sourceParts,
  };
}

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
