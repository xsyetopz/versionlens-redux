#!/usr/bin/env bun

import { readFileSync, writeFileSync } from "node:fs";

const nextVersion = Bun.argv[2];
const checkOnly = Bun.argv.includes("--check");
const semverPattern =
	/^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-([0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?(?:\+([0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?$/u;
const numericIdentifierPattern = /^\d+$/u;

function fail(message) {
	throw new Error(message);
}

function parseSemver(value) {
	if (
		typeof value !== "string" ||
		value.length === 0 ||
		value.trim() !== value
	) {
		fail(`invalid SemVer: ${JSON.stringify(value)}`);
	}
	const match = semverPattern.exec(value);
	if (!match) {
		fail(`invalid SemVer: ${JSON.stringify(value)}`);
	}
	const prerelease = match[4]?.split(".") ?? [];
	for (const identifier of prerelease) {
		if (
			numericIdentifierPattern.test(identifier) &&
			identifier.length > 1 &&
			identifier[0] === "0"
		) {
			fail(
				`invalid SemVer numeric prerelease identifier: ${JSON.stringify(identifier)}`,
			);
		}
	}
	return {
		major: BigInt(match[1]),
		minor: BigInt(match[2]),
		patch: BigInt(match[3]),
		prerelease,
	};
}

function compareSemver(left, right) {
	for (const field of ["major", "minor", "patch"]) {
		if (left[field] !== right[field]) {
			return left[field] < right[field] ? -1 : 1;
		}
	}
	return comparePrerelease(left.prerelease, right.prerelease);
}

function comparePrerelease(left, right) {
	if (left.length === 0 || right.length === 0) {
		return comparePrereleasePresence(left.length, right.length);
	}
	return comparePrereleaseIdentifiers(left, right);
}

function comparePrereleasePresence(leftLength, rightLength) {
	return leftLength === rightLength ? 0 : leftLength === 0 ? 1 : -1;
}

function comparePrereleaseIdentifiers(left, right) {
	const length = Math.max(left.length, right.length);
	for (let index = 0; index < length; index += 1) {
		const a = left[index];
		const b = right[index];
		if (a === undefined || b === undefined) {
			return a === b ? 0 : a === undefined ? -1 : 1;
		}
		const comparison = comparePrereleaseIdentifier(a, b);
		if (comparison !== 0) {
			return comparison;
		}
	}
	return 0;
}

function comparePrereleaseIdentifier(a, b) {
	if (a === b) {
		return 0;
	}
	const aNumeric = numericIdentifierPattern.test(a);
	const bNumeric = numericIdentifierPattern.test(b);
	if (aNumeric && bNumeric) {
		return BigInt(a) < BigInt(b) ? -1 : 1;
	}
	if (aNumeric !== bNumeric) {
		return aNumeric ? -1 : 1;
	}
	return a < b ? -1 : 1;
}

function replaceExactly(source, pattern, replacement, expectedCount, path) {
	let count = 0;
	const output = source.replace(pattern, (...args) => {
		count += 1;
		return typeof replacement === "function"
			? replacement(...args)
			: replacement;
	});
	if (count !== expectedCount) {
		fail(
			`${path}: expected ${expectedCount} version occurrence(s), found ${count}`,
		);
	}
	return output;
}

function currentVersion(path, pattern) {
	const match = pattern.exec(readFileSync(path, "utf8"));
	if (!match?.groups?.version) {
		fail(`${path}: could not read the current version`);
	}
	parseSemver(match.groups.version);
	return match.groups.version;
}

if (!nextVersion) {
	fail("usage: bun scripts/bump-version.mjs <major.minor.patch> [--check]");
}
const parsedNext = parseSemver(nextVersion);
const rootPattern =
	/"name": "@versionlens\/workspace",[\s\S]*?"version": "(?<version>[^"]+)"/u;
const previousVersion = currentVersion("package.json", rootPattern);
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
for (const [path, pattern] of manifests) {
	const version = currentVersion(path, pattern);
	if (version !== previousVersion) {
		fail(
			`${path}: version ${version} is not synchronized with ${previousVersion}`,
		);
	}
}

if (checkOnly) {
	console.log(`Verified synchronized release version ${nextVersion}.`);
	process.exit(0);
}

const changes = new Map();
function update(path, pattern, replacement, count = 1) {
	const source = changes.get(path) ?? readFileSync(path, "utf8");
	changes.set(path, replaceExactly(source, pattern, replacement, count, path));
}

update(
	"package.json",
	/("name": "@versionlens\/workspace",[\s\S]*?"version": ")[^"]+(")/u,
	(_match, prefix, suffix) => prefix + nextVersion + suffix,
);
update(
	"packages/vscode-extension/package.json",
	/("name": "versionlens-redux",[\s\S]*?"version": ")[^"]+(")/u,
	(_match, prefix, suffix) => prefix + nextVersion + suffix,
);
update(
	"Cargo.toml",
	/(\[workspace\.package\][\s\S]*?^version = ")[^"]+(")/mu,
	(_match, prefix, suffix) => prefix + nextVersion + suffix,
);

const rustPackages = [
	"versionlens-cache",
	"versionlens-core",
	"versionlens-edits",
	"versionlens-http",
	"versionlens-lsp",
	"versionlens-napi",
	"versionlens-parsers",
	"versionlens-providers",
	"versionlens-suggestions",
	"versionlens-versions",
	"versionlens-vscode-model",
];
for (const name of rustPackages) {
	update(
		"Cargo.lock",
		new RegExp(`(name = "${name}"\\nversion = ")[^"]+(")`, "u"),
		(_match, prefix, suffix) => prefix + nextVersion + suffix,
	);
}

update(
	"bun.lock",
	/("packages\/vscode-extension": \{\n\s+"name": "versionlens-redux",\n\s+"version": ")[^"]+(")/u,
	(_match, prefix, suffix) => prefix + nextVersion + suffix,
);
update(
	"packages/zed-extension/Cargo.toml",
	/(name = "versionlens-zed-extension"\nversion = ")[^"]+(")/u,
	(_match, prefix, suffix) => prefix + nextVersion + suffix,
);
update(
	"packages/zed-extension/Cargo.lock",
	/(name = "versionlens-zed-extension"\nversion = ")[^"]+(")/u,
	(_match, prefix, suffix) => prefix + nextVersion + suffix,
);
update(
	"packages/zed-extension/extension.toml",
	/^version = "[^"]+"$/mu,
	`version = "${nextVersion}"`,
);
update(
	"packages/jetbrains-plugin/build.gradle.kts",
	/^version = "[^"]+"$/mu,
	`version = "${nextVersion}"`,
);

for (const [path, contents] of changes) {
	writeFileSync(path, contents);
}
console.log(
	`Bumped ${changes.size} repository files from ${previousVersion} to ${nextVersion}.`,
);
