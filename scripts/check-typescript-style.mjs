#!/usr/bin/env bun

import fs from "node:fs";
import path from "node:path";

const roots = ["packages/vscode-extension/src", "tests"];
const ignoredDirs = new Set(["dist", "node_modules", "target"]);
const forbiddenPatterns = [
	{
		name: "TypeScript enum",
		pattern: /\benum\s+[A-Za-z_$][\w$]*/g,
	},
	{
		name: "TypeScript namespace",
		pattern: /\bnamespace\s+[A-Za-z_$][\w$]*/g,
	},
	{
		name: "TypeScript internal module",
		pattern: /\bmodule\s+[A-Za-z_$][\w$]*/g,
	},
	{
		name: "TypeScript import assignment",
		pattern: /\bimport\s+[A-Za-z_$][\w$]*\s*=\s*require\s*\(/g,
	},
	{
		name: "TypeScript constructor parameter property",
		pattern:
			/\bconstructor\s*\([^)]*\b(?:public|private|protected|readonly)\s+[A-Za-z_$]/gs,
	},
	{
		name: "TypeScript decorator",
		pattern: /^\s*@[A-Za-z_$][\w$]*/gm,
	},
];
const productionForbiddenPatterns = [
	{
		name: "production catch-all Record<string, ...> shape",
		pattern: /\bRecord\s*<\s*string\s*,/g,
	},
	{
		name: "production string index signature",
		pattern: /\[\s*[A-Za-z_$][\w$]*\s*:\s*string\s*\]\s*:/g,
	},
	{
		name: "production inline object type predicate",
		pattern: /\bis\s*\{/g,
	},
];
const offenders = [];
let checked = 0;

function stripComments(source) {
	return source
		.replaceAll(/\/\*[\s\S]*?\*\//g, (match) => whitespaceFor(match))
		.replaceAll(/\/\/.*$/gm, (match) => whitespaceFor(match));
}

function whitespaceFor(source) {
	return source.replaceAll(/[^\n]/g, " ");
}

function lineNumber(source, index) {
	return source.slice(0, index).split("\n").length;
}

function isTestFile(filePath) {
	return (
		filePath.endsWith(".test.ts") ||
		filePath.endsWith(".test-support.ts") ||
		filePath.includes(`${path.sep}tests${path.sep}`)
	);
}

function recordMatches(filePath, source, patterns) {
	for (const { name, pattern } of patterns) {
		for (const match of source.matchAll(pattern)) {
			offenders.push({
				filePath,
				line: lineNumber(source, match.index),
				name,
			});
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

		if (path.extname(entry.name) !== ".ts") {
			continue;
		}

		checked += 1;
		const source = fs.readFileSync(filePath, "utf8");
		const searchable = stripComments(source);
		recordMatches(filePath, searchable, forbiddenPatterns);
		if (!isTestFile(filePath)) {
			recordMatches(filePath, searchable, productionForbiddenPatterns);
		}
	}
}

for (const root of roots) {
	walk(root);
}

if (checked === 0) {
	console.error("checked 0 TypeScript files");
	process.exit(1);
}

if (offenders.length > 0) {
	for (const offender of offenders) {
		console.error(
			`${offender.filePath}:${offender.line} uses ${offender.name}`,
		);
	}
	process.exit(1);
}

console.log(
	`checked ${checked} TypeScript files; TypeScript stays enum-free, erasable, and free of production catch-all shapes`,
);
