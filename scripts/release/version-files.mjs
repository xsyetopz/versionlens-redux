import { readFileSync, writeFileSync } from "node:fs";

import { fail, parseSemver, replaceExactly } from "./semver.mjs";

const ROOT_VERSION_PATTERN =
  /"name": "@versionlens\/workspace",[\s\S]*?"version": "(?<version>[^"]+)"/u;
const ROOT_REPLACEMENT_PATTERN =
  /(?<prefix>"name": "@versionlens\/workspace",[\s\S]*?"version": ")[^"]+(?<suffix>")/u;
const EXTENSION_REPLACEMENT_PATTERN =
  /(?<prefix>"name": "versionlens-redux",[\s\S]*?"version": ")[^"]+(?<suffix>")/u;
const CARGO_REPLACEMENT_PATTERN =
  /(?<prefix>\[workspace\.package\][\s\S]*?^version = ")[^"]+(?<suffix>")/mu;
const BUN_LOCK_REPLACEMENT_PATTERN =
  /(?<prefix>"packages\/vscode-extension": \{\n\s+"name": "versionlens-redux",\n\s+"version": ")[^"]+(?<suffix>")/u;
const ZED_CARGO_REPLACEMENT_PATTERN =
  /(?<prefix>name = "versionlens-zed-extension"\nversion = ")[^"]+(?<suffix>")/u;
const VERSION_LINE_PATTERN = /^version = "[^"]+"$/mu;

const manifests = [
  [
    "Cargo.toml",
    /\[workspace\.package\][\s\S]*?^version = "(?<version>[^"]+)"/mu,
  ],
  [
    "packages/vscode-extension/package.json",
    /"name": "versionlens-redux",[\s\S]*?"version": "(?<version>[^"]+)"/u,
  ],
  [
    "packages/zed-extension/Cargo.toml",
    /name = "versionlens-zed-extension"\nversion = "(?<version>[^"]+)"/u,
  ],
  [
    "packages/zed-extension/extension.toml",
    /^version = "(?<version>[^"]+)"$/mu,
  ],
  [
    "packages/jetbrains-plugin/build.gradle.kts",
    /^version = "(?<version>[^"]+)"$/mu,
  ],
];

function currentVersion(path, pattern) {
  const match = pattern.exec(readFileSync(path, "utf8"));
  if (!match?.groups?.version) {
    fail(`${path}: could not read the current version`);
  }
  parseSemver(match.groups.version);
  return match.groups.version;
}

function readRepositoryVersion() {
  return currentVersion("package.json", ROOT_VERSION_PATTERN);
}

function rustWorkspacePackages() {
  const metadata = Bun.spawnSync([
    "cargo",
    "metadata",
    "--no-deps",
    "--format-version",
    "1",
  ]);
  if (metadata.exitCode !== 0) {
    fail(`cargo metadata failed: ${metadata.stderr.toString()}`);
  }
  const workspace = JSON.parse(metadata.stdout.toString());
  const workspaceMembers = new Set(workspace.workspace_members);
  return workspace.packages
    .filter(({ id }) => workspaceMembers.has(id))
    .map(({ name }) => name)
    .toSorted((left, right) => left.localeCompare(right));
}

function assertSynchronizedVersions(previousVersion, rustPackages) {
  for (const [path, pattern] of manifests) {
    const version = currentVersion(path, pattern);
    if (version !== previousVersion) {
      fail(
        `${path}: version ${version} is not synchronized with ${previousVersion}`,
      );
    }
  }

  for (const name of rustPackages) {
    const version = currentVersion(
      "Cargo.lock",
      new RegExp(`name = "${name}"\\nversion = "(?<version>[^"]+)"`, "u"),
    );
    if (version !== previousVersion) {
      fail(
        `Cargo.lock: ${name} version ${version} is not synchronized with ${previousVersion}`,
      );
    }
  }
}

function updateRepositoryVersions(nextVersion, rustPackages) {
  const changes = new Map();
  function applyUpdate(path, pattern, replacement, count = 1) {
    const source = changes.get(path) ?? readFileSync(path, "utf8");
    changes.set(
      path,
      replaceExactly({
        expectedCount: count,
        path,
        pattern,
        replacement,
        source,
      }),
    );
  }

  const replaceVersion = (_match, prefix, suffix) =>
    prefix + nextVersion + suffix;
  applyCoreManifestUpdates(applyUpdate, replaceVersion);
  applyRustLockUpdates(applyUpdate, replaceVersion, rustPackages);
  applyPackageManifestUpdates(applyUpdate, replaceVersion, nextVersion);

  for (const [path, contents] of changes) {
    writeFileSync(path, contents);
  }
  return changes.size;
}

function applyCoreManifestUpdates(applyUpdate, replaceVersion) {
  applyUpdate("package.json", ROOT_REPLACEMENT_PATTERN, replaceVersion);
  applyUpdate(
    "packages/vscode-extension/package.json",
    EXTENSION_REPLACEMENT_PATTERN,
    replaceVersion,
  );
  applyUpdate("Cargo.toml", CARGO_REPLACEMENT_PATTERN, replaceVersion);
}

function applyRustLockUpdates(applyUpdate, replaceVersion, rustPackages) {
  for (const name of rustPackages) {
    applyUpdate(
      "Cargo.lock",
      new RegExp(
        `(?<prefix>name = "${name}"\\nversion = ")[^"]+(?<suffix>")`,
        "u",
      ),
      replaceVersion,
    );
  }
}

function applyPackageManifestUpdates(applyUpdate, replaceVersion, nextVersion) {
  applyUpdate("bun.lock", BUN_LOCK_REPLACEMENT_PATTERN, replaceVersion);
  applyUpdate(
    "packages/zed-extension/Cargo.toml",
    ZED_CARGO_REPLACEMENT_PATTERN,
    replaceVersion,
  );
  applyUpdate(
    "packages/zed-extension/Cargo.lock",
    ZED_CARGO_REPLACEMENT_PATTERN,
    replaceVersion,
  );
  applyUpdate(
    "packages/zed-extension/extension.toml",
    VERSION_LINE_PATTERN,
    `version = "${nextVersion}"`,
  );
  applyUpdate(
    "packages/jetbrains-plugin/build.gradle.kts",
    VERSION_LINE_PATTERN,
    `version = "${nextVersion}"`,
  );
}

export {
  assertSynchronizedVersions,
  readRepositoryVersion,
  rustWorkspacePackages,
  updateRepositoryVersions,
};
