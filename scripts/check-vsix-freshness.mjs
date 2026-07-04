#!/usr/bin/env bun

import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

const packageRoot = "packages/vscode-extension";
const packagePath = path.join(packageRoot, "package.json");
const packageJson = JSON.parse(fs.readFileSync(packagePath, "utf8"));
const vsixPath = path.join(
	packageRoot,
	`${packageJson.name}-${packageJson.version}.vsix`,
);
const offenders = [];
const expectedEntries = [
	"[Content_Types].xml",
	"extension.vsixmanifest",
	"extension/LICENSE.txt",
	"extension/dist/extension.js",
	"extension/images/faq/ext-host-log.jpg",
	"extension/images/faq/ext-log.jpg",
	"extension/images/faq/show-prereleases.gif",
	"extension/images/faq/show-releases.gif",
	"extension/images/icons/tag-active.svg",
	"extension/images/icons/tag-inactive-light.svg",
	"extension/images/icons/tag-inactive.svg",
	"extension/images/icons/toggle-active-progress.svg",
	"extension/images/icons/toggle-active.svg",
	"extension/images/icons/toggle-death.svg",
	"extension/images/icons/toggle-inactive-light.svg",
	"extension/images/icons/toggle-inactive.svg",
	"extension/images/logo.png",
	"extension/native/versionlens_napi.node",
	"extension/package.json",
	"extension/readme.md",
	"extension/src/schema/versionlens.multi-registries.json",
];

function stableJson(value) {
	return JSON.stringify(value);
}

function vsixEntry(entryPath) {
	return execFileSync("unzip", ["-p", vsixPath, `extension/${entryPath}`], {
		encoding: "utf8",
	});
}

function vsixBinaryEntry(entryPath) {
	return execFileSync("unzip", ["-p", vsixPath, `extension/${entryPath}`], {
		maxBuffer: 64 * 1024 * 1024,
	});
}

function digest(buffer) {
	return createHash("sha256").update(buffer).digest("hex");
}

function compareField(label, current, packaged) {
	if (stableJson(current) !== stableJson(packaged)) {
		offenders.push(`VSIX package.json ${label} is stale`);
	}
}

function compareTextEntry(entryPath) {
	const current = fs.readFileSync(path.join(packageRoot, entryPath), "utf8");
	const packaged = vsixEntry(entryPath);
	if (current !== packaged) {
		offenders.push(`VSIX ${entryPath} is stale`);
	}
}

function compareBinaryEntry(entryPath) {
	const current = fs.readFileSync(path.join(packageRoot, entryPath));
	const packaged = vsixBinaryEntry(entryPath);
	if (digest(current) !== digest(packaged)) {
		offenders.push(`VSIX ${entryPath} is stale`);
	}
}

if (fs.existsSync(vsixPath)) {
	const entries = execFileSync("unzip", ["-Z1", vsixPath], {
		encoding: "utf8",
	})
		.split(/\r?\n/u)
		.filter(Boolean)
		.sort();
	if (stableJson(entries) !== stableJson(expectedEntries.toSorted())) {
		offenders.push("VSIX packaged entry set differs from expected payload");
	}
	const packagedPackage = JSON.parse(vsixEntry("package.json"));
	compareField("main", packageJson.main, packagedPackage.main);
	compareField(
		"activationEvents",
		packageJson.activationEvents,
		packagedPackage.activationEvents,
	);
	compareField(
		"contributes.commands",
		packageJson.contributes?.commands,
		packagedPackage.contributes?.commands,
	);
	compareField(
		"contributes.menus",
		packageJson.contributes?.menus,
		packagedPackage.contributes?.menus,
	);
	compareField(
		"contributes.configuration",
		packageJson.contributes?.configuration,
		packagedPackage.contributes?.configuration,
	);
	compareTextEntry("dist/extension.js");
	compareBinaryEntry("native/versionlens_napi.node");
}

if (offenders.length > 0) {
	console.error(offenders.join("\n"));
	process.exit(1);
}

console.log(
	"checked VSIX freshness against current manifest and runtime payload",
);
