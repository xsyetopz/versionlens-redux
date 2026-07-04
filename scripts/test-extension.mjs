#!/usr/bin/env bun
import { spawnSync } from "node:child_process";
import { readdirSync, statSync } from "node:fs";
import { join } from "node:path";

const testRoot = "packages/vscode-extension/src";

function collectTestFiles(directory) {
	const files = [];
	for (const entry of readdirSync(directory).sort()) {
		const path = join(directory, entry);
		if (statSync(path).isDirectory()) {
			files.push(...collectTestFiles(path));
		} else if (path.endsWith(".test.ts")) {
			files.push(path);
		}
	}
	return files;
}

const testFiles = collectTestFiles(testRoot);
for (const file of testFiles) {
	const result = spawnSync("bun", ["test", file], {
		stdio: "inherit",
	});
	if (result.status !== 0) {
		process.exit(result.status ?? 1);
	}
}
