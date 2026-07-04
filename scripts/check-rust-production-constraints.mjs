#!/usr/bin/env bun

import fs from "node:fs";
import path from "node:path";

const roots = ["crates"];
const ignoredDirs = new Set(["target"]);
const productionForbiddenPatterns = [
	{ name: "production .clone() call", pattern: /\.clone\s*\(/g },
	{ name: "production .cloned() call", pattern: /\.cloned\s*\(/g },
	{ name: "production .unwrap() call", pattern: /\.unwrap\s*\(/g },
	{ name: "production .expect() call", pattern: /\.expect\s*\(/g },
	{ name: "production panic! macro", pattern: /\bpanic\s*!/g },
	{ name: "production std::panic use", pattern: /\bstd::panic::/g },
	{ name: "production todo! macro", pattern: /\btodo\s*!/g },
	{ name: "production unimplemented! macro", pattern: /\bunimplemented\s*!/g },
	{ name: "production dbg! macro", pattern: /\bdbg\s*!/g },
	{ name: "production println! macro", pattern: /\bprintln\s*!/g },
	{ name: "production eprintln! macro", pattern: /\beprintln\s*!/g },
	{
		name: "manual trait implementation",
		pattern:
			/^\s*impl(?:\s*<[^>{}]*>)?\s+(?:unsafe\s+)?[A-Za-z_][\w:<>]*(?:\s*<[^>{}]*>)?\s+for\s+/gm,
	},
];
const allRustForbiddenPatterns = [
	{
		name: "user-defined Rust macro",
		pattern: /macro_rules!|#\s*\[\s*macro_export\s*\]/g,
	},
];
const offenders = [];
let checked = 0;

function isTestFile(filePath) {
	const parts = filePath.split(path.sep);
	const fileName = parts.at(-1) ?? "";
	return (
		parts.includes("tests") ||
		fileName === "tests.rs" ||
		fileName.endsWith("_tests.rs")
	);
}

function lineNumber(source, index) {
	return source.slice(0, index).split("\n").length;
}

function recordMatches(filePath, source, patterns) {
	for (const { name, pattern } of patterns) {
		for (const match of source.matchAll(pattern)) {
			offenders.push({ filePath, line: lineNumber(source, match.index), name });
		}
	}
}

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

		if (path.extname(entry.name) !== ".rs") {
			continue;
		}

		checked += 1;
		const source = fs.readFileSync(filePath, "utf8");
		recordMatches(filePath, source, allRustForbiddenPatterns);
		if (!isTestFile(filePath)) {
			recordMatches(filePath, source, productionForbiddenPatterns);
		}
	}
}

for (const root of roots) {
	walk(root);
}

if (checked === 0) {
	console.error("checked 0 Rust files");
	process.exit(1);
}

if (offenders.length > 0) {
	for (const offender of offenders) {
		console.error(`${offender.filePath}:${offender.line} has ${offender.name}`);
	}
	process.exit(1);
}

console.log(
	`checked ${checked} Rust files; production code avoids forbidden calls/macros, manual trait impls, and user macros`,
);
