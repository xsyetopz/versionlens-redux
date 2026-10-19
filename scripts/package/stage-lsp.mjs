#!/usr/bin/env bun
import { createHash } from "node:crypto";
import {
  chmodSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readdirSync,
  readFileSync,
  renameSync,
  rmSync,
} from "node:fs";
import { join } from "node:path";
import process from "node:process";

const targets = JSON.parse(
  readFileSync(new URL("./targets.json", import.meta.url), "utf8"),
).targets;

function rustTargetFor(vscodeTarget) {
  const entry = targets.find(({ vscode }) => vscode === vscodeTarget);
  if (!entry) {
    throw new Error(`unsupported VS Code target: ${vscodeTarget}`);
  }
  return entry.rust;
}

function executableNameFor(rustTarget) {
  return rustTarget.includes("windows")
    ? "versionlens-lsp.exe"
    : "versionlens-lsp";
}

function extract(archive, destination) {
  let command = ["tar", "-xzf", archive, "-C", destination];
  if (archive.endsWith(".zip")) {
    command = ["unzip", "-o", "-q", archive, "-d", destination];
    if (process.platform === "win32") {
      const root = process.env.SystemRoot ?? "C:\\Windows";
      command = [
        join(root, "System32", "tar.exe"),
        "-xf",
        archive,
        "-C",
        destination,
      ];
    }
  }
  const result = Bun.spawnSync(command, { stderr: "pipe", stdout: "pipe" });
  if (result.exitCode !== 0) {
    throw new Error(
      `Could not extract ${archive}: ${new TextDecoder().decode(result.stderr)}`,
    );
  }
}

function verifyChecksum(archivesDir, archiveName) {
  const sums = join(archivesDir, "SHA256SUMS");
  if (!existsSync(sums)) {
    return;
  }
  const line = readFileSync(sums, "utf8")
    .split("\n")
    .find((entry) => entry.trim().endsWith(` ${archiveName}`));
  if (!line) {
    throw new Error(`SHA256SUMS has no entry for ${archiveName}`);
  }
  const expected = line.trim().split(/\s+/u)[0];
  const actual = createHash("sha256")
    .update(readFileSync(join(archivesDir, archiveName)))
    .digest("hex");
  if (expected !== actual) {
    throw new Error(
      `${archiveName} checksum mismatch: expected ${expected}, got ${actual}`,
    );
  }
}

/**
 * Extract the build-lsp binary for `rustTarget` from `archivesDir` into
 * `destination` and return its path. Verifies against SHA256SUMS when present.
 */
function stageLspBinary(archivesDir, rustTarget, destination) {
  const archiveName = readdirSync(archivesDir).find(
    (name) =>
      name.startsWith("versionlens-lsp-") &&
      (name.endsWith(`-${rustTarget}.tar.gz`) ||
        name.endsWith(`-${rustTarget}.zip`)),
  );
  if (!archiveName) {
    throw new Error(`No build-lsp archive for ${rustTarget} in ${archivesDir}`);
  }
  verifyChecksum(archivesDir, archiveName);
  const executableName = executableNameFor(rustTarget);
  // Scratch lives under the destination so the rename never crosses volumes.
  mkdirSync(destination, { recursive: true });
  const scratch = mkdtempSync(join(destination, ".extract-"));
  try {
    extract(join(archivesDir, archiveName), scratch);
    const extracted = join(scratch, executableName);
    if (!existsSync(extracted)) {
      throw new Error(`${archiveName} does not contain ${executableName}`);
    }
    const staged = join(destination, executableName);
    rmSync(staged, { force: true });
    renameSync(extracted, staged);
    if (!rustTarget.includes("windows")) {
      chmodSync(staged, 0o755);
    }
    return staged;
  } finally {
    rmSync(scratch, { force: true, recursive: true });
  }
}

export { executableNameFor, rustTargetFor, stageLspBinary };

if (import.meta.main) {
  const [, , archivesDir, rustTarget, destination] = Bun.argv;
  if (!(archivesDir && rustTarget && destination)) {
    throw new Error(
      "usage: bun scripts/package/stage-lsp.mjs <archives-dir> <rust-target> <destination-dir>",
    );
  }
  console.log(`Staged ${stageLspBinary(archivesDir, rustTarget, destination)}`);
}
