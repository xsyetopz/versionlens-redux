#!/usr/bin/env bun

import fs from "node:fs";
import path from "node:path";

const roots = ["crates"];
const ignoredDirs = new Set(["target"]);
const inlineTestModulePattern =
	/#\[cfg\s*\(\s*test\s*\)\]\s*(?:#\[[^\]]*\]\s*)*mod\s+[A-Za-z_][A-Za-z0-9_]*\s*\{/g;
const pathAttributePattern = /^\s*#\[path\s*=/gm;
const testFunctionPattern = /^\s*#\[test\]/gm;
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

function recordSourceMatches(filePath, source, pattern, message) {
	for (const match of source.matchAll(pattern)) {
		offenders.push({
			filePath,
			line: lineNumber(source, match.index),
			message,
		});
	}
}

function hasSuffixedTestCategory(filePath) {
	const parts = filePath.split(path.sep);
	const fileName = parts.at(-1) ?? "";

	return (
		fileName.endsWith("_tests.rs") ||
		parts.some((part) => part.endsWith("_tests"))
	);
}

function checkTestCategoryName(filePath) {
	if (!(isTestFile(filePath) && hasSuffixedTestCategory(filePath))) {
		return;
	}

	offenders.push({
		filePath,
		line: 1,
		message:
			"uses a suffixed test category name; use tests/<category>.rs or tests/<category>/",
	});
}

function checkRustFile(filePath) {
	checked += 1;
	const source = fs.readFileSync(filePath, "utf8");
	recordSourceMatches(
		filePath,
		source,
		pathAttributePattern,
		"uses #[path] module wiring instead of adjacent test modules",
	);
	recordSourceMatches(
		filePath,
		source,
		inlineTestModulePattern,
		"has an inline Rust test module",
	);

	if (!isTestFile(filePath)) {
		recordSourceMatches(
			filePath,
			source,
			testFunctionPattern,
			"has a Rust test outside a test file",
		);
	}

	checkTestCategoryName(filePath);
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

		checkRustFile(filePath);
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
		console.error(`${offender.filePath}:${offender.line} ${offender.message}`);
	}
	process.exit(1);
}

console.log(
	`checked ${checked} Rust files; no inline Rust tests or #[path] test wiring`,
);
