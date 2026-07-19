#!/usr/bin/env bun

import { chmodSync, copyFileSync, mkdirSync, rmSync } from "node:fs";
import { join } from "node:path";
import process, {
  arch as hostArchitecture,
  platform as hostPlatform,
} from "node:process";

const [requestedPlatform, requestedArchitecture, rustTarget] =
  Bun.argv.slice(2);
const requestedTargetPackaging =
  requestedPlatform !== undefined ||
  requestedArchitecture !== undefined ||
  rustTarget !== undefined;
if (
  requestedTargetPackaging &&
  !(requestedPlatform && requestedArchitecture && rustTarget)
) {
  throw new Error(
    "Target packaging requires platform, architecture, and Rust target arguments.",
  );
}
if (
  requestedTargetPackaging &&
  (requestedPlatform !== hostPlatform ||
    requestedArchitecture !== hostArchitecture)
) {
  throw new Error(
    `Cannot package ${requestedPlatform}-${requestedArchitecture} on ${hostPlatform}-${hostArchitecture}; use a native runner.`,
  );
}

const platform = requestedPlatform ?? hostPlatform;
const architecture = requestedArchitecture ?? hostArchitecture;
const supportedTargets = new Map([
  ["linux-x64", "x86_64-unknown-linux-gnu"],
  ["linux-arm64", "aarch64-unknown-linux-gnu"],
  ["darwin-x64", "x86_64-apple-darwin"],
  ["darwin-arm64", "aarch64-apple-darwin"],
  ["win32-x64", "x86_64-pc-windows-msvc"],
  ["win32-arm64", "aarch64-pc-windows-msvc"],
]);
const editorTarget = `${platform}-${architecture}`;
const expectedRustTarget = supportedTargets.get(editorTarget);
if (!expectedRustTarget) {
  throw new Error(`Unsupported Zed package target: ${editorTarget}`);
}
if (rustTarget && rustTarget !== expectedRustTarget) {
  throw new Error(
    `${editorTarget} requires Rust target ${expectedRustTarget}, received ${rustTarget}.`,
  );
}

let executableName = "versionlens-lsp";
if (platform === "win32") {
  executableName = "versionlens-lsp.exe";
}
const sourceParts = ["target"];
if (rustTarget) {
  sourceParts.push(rustTarget);
}
sourceParts.push("release", executableName);
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
