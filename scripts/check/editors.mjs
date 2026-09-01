#!/usr/bin/env bun
import { createHash } from "node:crypto";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import process, { arch, platform } from "node:process";

const linePattern = /\r?\n/u;
const { version } = JSON.parse(readFileSync("package.json", "utf8"));
let executableName = "versionlens-lsp";
if (platform === "win32") {
  executableName = "versionlens-lsp.exe";
}
const vscodeArtifact = join(
  "packages",
  "vscode-extension",
  `versionlens-redux-${version}.vsix`,
);
const zedArtifact = join(
  "dist",
  `versionlens-redux-zed-extension-${platform}-${arch}.tar.gz`,
);
const neovimArtifact = join(
  "dist",
  `versionlens-redux-neovim-plugin-${platform}-${arch}.tar.gz`,
);
const jetbrainsPlatform =
  platform === "darwin" ? "mac" : platform === "win32" ? "windows" : platform;
const jetbrainsArchitecture = arch === "x64" ? "x86_64" : arch;
const jetbrainsArtifact = join(
  "packages",
  "jetbrains-plugin",
  "build",
  "distributions",
  `versionlens-jetbrains-plugin-${version}-${jetbrainsPlatform}-${jetbrainsArchitecture}.zip`,
);

function run(command) {
  const result = Bun.spawnSync(command);
  if (result.exitCode !== 0) {
    process.stderr.write(result.stderr);
    process.exit(result.exitCode);
  }
  return result.stdout;
}

function requireEntry(entries, expected, artifact) {
  if (!entries.split(linePattern).includes(expected)) {
    throw new Error(`${artifact} does not contain ${expected}`);
  }
}

function digest(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

function requireMatchingBinary(actual, expectedPath, artifact) {
  if (digest(actual) !== digest(readFileSync(expectedPath))) {
    throw new Error(`${artifact} contains a stale runtime binary`);
  }
}

const vscodeEntries = new TextDecoder().decode(
  run(["unzip", "-Z1", vscodeArtifact]),
);
requireEntry(
  vscodeEntries,
  "extension/native/versionlens_napi.node",
  vscodeArtifact,
);
requireMatchingBinary(
  run([
    "unzip",
    "-p",
    vscodeArtifact,
    "extension/native/versionlens_napi.node",
  ]),
  join("packages", "vscode-extension", "native", "versionlens_napi.node"),
  vscodeArtifact,
);

const zedEntries = new TextDecoder().decode(run(["tar", "-tzf", zedArtifact]));
requireEntry(zedEntries, `bin/${executableName}`, zedArtifact);
requireMatchingBinary(
  run(["tar", "-xOzf", zedArtifact, `bin/${executableName}`]),
  join("target", "release", executableName),
  zedArtifact,
);

const neovimBinary = `bin/${platform}-${arch}/${executableName}`;
const neovimEntries = new TextDecoder().decode(
  run(["tar", "-tzf", neovimArtifact]),
);
requireEntry(neovimEntries, neovimBinary, neovimArtifact);
requireEntry(neovimEntries, "lua/versionlens/init.lua", neovimArtifact);
requireEntry(neovimEntries, "doc/versionlens.txt", neovimArtifact);
requireMatchingBinary(
  run(["tar", "-xOzf", neovimArtifact, neovimBinary]),
  join("target", "release", executableName),
  neovimArtifact,
);

const temporaryDirectory = mkdtempSync(
  join(tmpdir(), "versionlens-jetbrains-"),
);
try {
  const outerEntries = new TextDecoder().decode(
    run(["unzip", "-Z1", jetbrainsArtifact]),
  );
  const pluginRoot = "versionlens-jetbrains-plugin";
  const pluginJar = outerEntries
    .split(linePattern)
    .find((entry) =>
      entry.match(/\/lib\/versionlens-jetbrains-plugin-[^/]+\.jar$/u),
    );
  if (!pluginJar) {
    throw new Error(`${jetbrainsArtifact} does not contain the plugin JAR`);
  }

  const binaryEntry = `${pluginRoot}/bin/${executableName}`;
  requireEntry(outerEntries, binaryEntry, jetbrainsArtifact);
  requireMatchingBinary(
    run(["unzip", "-p", jetbrainsArtifact, binaryEntry]),
    join("target", "release", executableName),
    jetbrainsArtifact,
  );

  const jarPath = join(temporaryDirectory, "plugin.jar");
  writeFileSync(jarPath, run(["unzip", "-p", jetbrainsArtifact, pluginJar]));
  const descriptor = new TextDecoder().decode(
    run(["unzip", "-p", jarPath, "META-INF/plugin.xml"]),
  );
  for (const module of [
    `com.intellij.modules.os.${jetbrainsPlatform}`,
    `com.intellij.modules.arch.${jetbrainsArchitecture}`,
  ]) {
    if (!descriptor.includes(`<depends>${module}</depends>`)) {
      throw new Error(`${jetbrainsArtifact} does not declare ${module}`);
    }
  }
} finally {
  rmSync(temporaryDirectory, { force: true, recursive: true });
}

console.log("Verified bundled runtimes in all four editor packages.");
