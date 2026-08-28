#!/usr/bin/env bun
import process from "node:process";

import { compareSemver, fail, parseSemver } from "./semver.mjs";
import {
  assertSynchronizedVersions,
  readRepositoryVersion,
  rustWorkspacePackages,
  updateRepositoryVersions,
} from "./version-files.mjs";

const [, , nextVersion] = Bun.argv;
const checkOnly = Bun.argv.includes("--check");

if (!nextVersion) {
  fail("usage: bun scripts/release/version.mjs <major.minor.patch> [--check]");
}

const parsedNext = parseSemver(nextVersion);
const previousVersion = readRepositoryVersion();
const rustPackages = rustWorkspacePackages();
assertSynchronizedVersions(previousVersion, rustPackages);

if (checkOnly && nextVersion !== previousVersion) {
  fail(`requested version ${nextVersion} does not match ${previousVersion}`);
}
if (
  !checkOnly &&
  compareSemver(parsedNext, parseSemver(previousVersion)) <= 0
) {
  fail(
    `new version ${nextVersion} must have higher SemVer precedence than ${previousVersion}`,
  );
}

if (checkOnly) {
  console.log(`Verified synchronized release version ${nextVersion}.`);
  process.exit(0);
}

const changedFiles = updateRepositoryVersions(nextVersion, rustPackages);
console.log(
  `Bumped ${changedFiles} repository files from ${previousVersion} to ${nextVersion}.`,
);
