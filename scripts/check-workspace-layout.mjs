#!/usr/bin/env bun

import fs from "node:fs";

const expectedCrates = [
	"versionlens-cache",
	"versionlens-core",
	"versionlens-edits",
	"versionlens-http",
	"versionlens-napi",
	"versionlens-parsers",
	"versionlens-providers",
	"versionlens-suggestions",
	"versionlens-versions",
	"versionlens-vscode-model",
];
const expectedPackages = ["vscode-extension"];
const workspacePackageFields = [
	"edition",
	"rust-version",
	"license",
	"repository",
	"readme",
	"keywords",
	"categories",
	"version",
];
const expectedInternalDependencies = {
	"versionlens-cache": [],
	"versionlens-core": [
		"versionlens-cache",
		"versionlens-edits",
		"versionlens-http",
		"versionlens-parsers",
		"versionlens-providers",
		"versionlens-suggestions",
		"versionlens-versions",
		"versionlens-vscode-model",
	],
	"versionlens-edits": [
		"versionlens-parsers",
		"versionlens-suggestions",
		"versionlens-versions",
		"versionlens-vscode-model",
	],
	"versionlens-http": [],
	"versionlens-napi": [
		"versionlens-core",
		"versionlens-http",
		"versionlens-parsers",
		"versionlens-vscode-model",
	],
	"versionlens-parsers": ["versionlens-vscode-model"],
	"versionlens-providers": ["versionlens-parsers", "versionlens-versions"],
	"versionlens-suggestions": ["versionlens-parsers", "versionlens-versions"],
	"versionlens-versions": [],
	"versionlens-vscode-model": [],
};
const offenders = [];

function childDirs(dir) {
	return fs
		.readdirSync(dir, { withFileTypes: true })
		.filter((entry) => entry.isDirectory())
		.map((entry) => entry.name)
		.sort();
}

function sameList(actual, expected) {
	return (
		actual.length === expected.length &&
		actual.every((value, index) => value === expected[index])
	);
}

function expectList(label, actual, expected) {
	if (!sameList(actual, expected)) {
		offenders.push(
			`${label}: expected ${expected.join(", ")}, got ${actual.join(", ")}`,
		);
	}
}

function manifestSection(source, name) {
	const start = source.indexOf(`[${name}]`);
	if (start < 0) {
		return "";
	}

	const next = source.indexOf("\n[", start + 1);
	return next < 0 ? source.slice(start) : source.slice(start, next);
}

function internalDependencies(source) {
	return [
		...manifestSection(source, "dependencies").matchAll(
			/^(versionlens-[\w-]+)\.workspace\s*=\s*true$/gmu,
		),
	]
		.map((match) => match[1])
		.sort();
}

expectList("crates", childDirs("crates"), expectedCrates);
expectList("packages", childDirs("packages"), expectedPackages);

if (fs.existsSync("src")) {
	offenders.push(
		"root src/ exists; product source must stay split under crates/ and packages/",
	);
}

const packageJson = JSON.parse(fs.readFileSync("package.json", "utf8"));
expectList("package.json workspaces", packageJson.workspaces ?? [], [
	"packages/*",
]);

const cargoToml = fs.readFileSync("Cargo.toml", "utf8");
const workspaceMembers =
	cargoToml
		.match(/members\s*=\s*\[([\s\S]*?)\]/u)?.[1]
		.match(/"([^"]+)"/g)
		?.map((member) => member.slice(1, -1))
		.sort() ?? [];
expectList(
	"Cargo.toml workspace members",
	workspaceMembers,
	expectedCrates.map((crate) => `crates/${crate}`).sort(),
);

for (const crateName of expectedCrates) {
	const manifestPath = `crates/${crateName}/Cargo.toml`;
	const manifest = fs.readFileSync(manifestPath, "utf8");
	if (!new RegExp(`^name\\s*=\\s*"${crateName}"$`, "mu").test(manifest)) {
		offenders.push(`${manifestPath} package name must match its directory`);
	}

	for (const field of workspacePackageFields) {
		if (
			!new RegExp(`^${field}\\.workspace\\s*=\\s*true$`, "mu").test(manifest)
		) {
			offenders.push(`${manifestPath} must inherit package.${field}`);
		}
	}

	if (!/^\[lints\]\s*\nworkspace\s*=\s*true$/mu.test(manifest)) {
		offenders.push(`${manifestPath} must inherit workspace lints`);
	}

	expectList(
		`${manifestPath} internal dependencies`,
		internalDependencies(manifest),
		expectedInternalDependencies[crateName],
	);
}

if (offenders.length > 0) {
	console.error(offenders.join("\n"));
	process.exit(1);
}

console.log("checked workspace layout; crates/ and packages/ stay split");
