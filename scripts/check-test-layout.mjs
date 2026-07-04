#!/usr/bin/env bun

import fs from "node:fs";
import path from "node:path";

const offenders = [];
const ignoredNames = new Set(["node_modules", "target", "dist"]);

function walk(dir, visit) {
	if (!fs.existsSync(dir)) {
		return;
	}

	for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
		if (ignoredNames.has(entry.name)) {
			continue;
		}

		const filePath = path.join(dir, entry.name);
		if (entry.isDirectory()) {
			walk(filePath, visit);
			continue;
		}

		visit(filePath);
	}
}

if (fs.existsSync("src/tests") || fs.existsSync("src/test")) {
	offenders.push(
		"root src/tests must not exist; keep Rust and TypeScript source split",
	);
}

walk("tests", (filePath) => {
	if (filePath.endsWith(".rs")) {
		offenders.push(`${filePath} is a Rust test in repo-level tests/`);
	}

	if (
		filePath.endsWith(".test.ts") &&
		!filePath.startsWith("tests/napi/") &&
		!filePath.startsWith("tests/e2e/")
	) {
		offenders.push(`${filePath} is not a repo-level boundary/e2e test`);
	}
});

if (offenders.length > 0) {
	console.error(offenders.join("\n"));
	process.exit(1);
}

console.log("checked test layout; repo-level tests stay boundary/e2e only");
