#!/usr/bin/env bun

import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const checkOnly = process.argv.includes("--check");
const extensionDist = checkOnly
	? mkdtempSync(join(tmpdir(), "versionlens-extension-build-"))
	: "packages/vscode-extension/dist";

if (!checkOnly) {
	rmSync(extensionDist, { force: true, recursive: true });
}

const result = await Bun.build({
	external: ["vscode"],
	entrypoints: ["packages/vscode-extension/src/extension.ts"],
	format: "cjs",
	minify: true,
	outdir: extensionDist,
	target: "node",
});

if (checkOnly) {
	rmSync(extensionDist, { force: true, recursive: true });
}

if (!result.success) {
	for (const log of result.logs) {
		console.error(log);
	}
	process.exit(1);
}
