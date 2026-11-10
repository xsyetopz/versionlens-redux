#!/usr/bin/env bun
import { compareSemver, fail, parseSemver } from "./semver.mjs";
import {
	assertSynchronizedVersions,
	readRepositoryVersion,
	rustWorkspacePackages,
	updateRepositoryVersions,
} from "./version-files.mjs";

const nextVersion = Bun.argv[2];
const checkOnly = Bun.argv.includes("--check");

if (nextVersion === undefined || nextVersion === "") {
	fail("usage: bun scripts/release/version.mjs <major.minor.patch> [--check]");
}

const parsedNext = parseSemver(nextVersion);
const previousVersion = await readRepositoryVersion();
const rustPackages = await rustWorkspacePackages();
await assertSynchronizedVersions(previousVersion, rustPackages);

if (checkOnly) {
	if (nextVersion !== previousVersion) {
		fail(`requested version ${nextVersion} does not match ${previousVersion}`);
	}
	console.log(`Verified synchronized release version ${nextVersion}.`);
} else {
	if (compareSemver(parsedNext, parseSemver(previousVersion)) <= 0) {
		fail(
			`new version ${nextVersion} must have higher SemVer precedence than ${previousVersion}`,
		);
	}
	const changedFiles = await updateRepositoryVersions(
		nextVersion,
		rustPackages,
	);
	console.log(
		`Bumped ${changedFiles} repository files from ${previousVersion} to ${nextVersion}.`,
	);
}
