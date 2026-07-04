#!/usr/bin/env bun

import fs from "node:fs";
import path from "node:path";

const MAX_LINES = 800;
const roots = ["crates", "packages/vscode-extension/src"];
const ignoredDirs = new Set(["dist", "node_modules", "target"]);
const sourceExtensions = new Set([".rs", ".ts"]);
const offenders = [];
let checked = 0;

function walk(dir) {
	if (!fs.existsSync(dir)) {
		return;
	}

	for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
		if (ignoredDirs.has(entry.name)) {
			continue;
		}

		const filePath = path.join(dir, entry.name);
		if (entry.isDirectory()) {
			walk(filePath);
			continue;
		}

		if (!sourceExtensions.has(path.extname(entry.name))) {
			continue;
		}

		checked += 1;
		const lines = fs.readFileSync(filePath, "utf8").split("\n").length;
		if (lines > MAX_LINES) {
			offenders.push({ filePath, lines });
		}
	}
}

for (const root of roots) {
	walk(root);
}

if (checked === 0) {
	console.error("checked 0 source files");
	process.exit(1);
}

if (offenders.length > 0) {
	for (const offender of offenders) {
		console.error(`${offender.filePath} has ${offender.lines} lines`);
	}
	process.exit(1);
}

console.log(`checked ${checked} source files; none over ${MAX_LINES} lines`);
