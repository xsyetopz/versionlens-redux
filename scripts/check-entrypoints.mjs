#!/usr/bin/env bun

import fs from "node:fs";
import path from "node:path";

const rustRoots = ["crates"];
const tsRoots = ["packages"];
const ignoredDirs = new Set(["dist", "node_modules", "target"]);
const rustLogicPattern =
	/^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?(?:fn|struct|enum|union|trait|type|const|static)\b|^\s*impl\b/gm;
const tsLogicPattern =
	/^\s*(?:export\s+)?(?:async\s+)?(?:function|class|interface|type|enum|const|let|var)\b/gm;
const offenders = [];
let checked = 0;

function lineNumber(source, index) {
	return source.slice(0, index).split("\n").length;
}

function checkFile(filePath, pattern, language) {
	checked += 1;
	const source = fs.readFileSync(filePath, "utf8");
	for (const match of source.matchAll(pattern)) {
		offenders.push({
			filePath,
			language,
			line: lineNumber(source, match.index),
			snippet: match[0].trim(),
		});
	}
}

function walk(dir, visit) {
	if (!fs.existsSync(dir)) {
		return;
	}

	for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
		if (ignoredDirs.has(entry.name)) {
			continue;
		}

		const filePath = path.join(dir, entry.name);
		if (entry.isDirectory()) {
			walk(filePath, visit);
			continue;
		}

		visit(filePath, entry.name);
	}
}

for (const root of rustRoots) {
	walk(root, (filePath, fileName) => {
		if (fileName === "lib.rs" || fileName === "main.rs") {
			checkFile(filePath, rustLogicPattern, "Rust");
		}
	});
}

for (const root of tsRoots) {
	walk(root, (filePath, fileName) => {
		if (fileName === "index.ts") {
			checkFile(filePath, tsLogicPattern, "TypeScript");
		}
	});
}

if (checked === 0) {
	console.error("checked 0 entrypoint files");
	process.exit(1);
}

if (offenders.length > 0) {
	for (const offender of offenders) {
		console.error(
			`${offender.filePath}:${offender.line} has ${offender.language} entrypoint logic: ${offender.snippet}`,
		);
	}
	process.exit(1);
}

console.log(
	`checked ${checked} entrypoint files; lib.rs, main.rs, and index.ts stay module/export-only`,
);
