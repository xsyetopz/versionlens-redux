#!/usr/bin/env bun
import {
  chmodSync,
  copyFileSync,
  mkdirSync,
  readFileSync,
  rmSync,
} from "node:fs";
import { join } from "node:path";
import process, {
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
