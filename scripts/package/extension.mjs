#!/usr/bin/env bun
// Bundles the VS Code extension; `--check` builds into a temporary folder to prove it compiles.
import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import process from "node:process";

const checkOnly = process.argv.includes("--check");
const outdir = checkOnly
	? await mkdtemp(join(tmpdir(), "versionlens-extension-"))
	: "packages/vscode-extension/dist";
await rm(outdir, { force: true, recursive: true });
const result = await Bun.build({
	entrypoints: ["packages/vscode-extension/src/extension.ts"],
	external: ["vscode"],
	format: "cjs",
	minify: true,
	outdir,
	target: "node"
});
if (checkOnly) {
	await rm(outdir, { force: true, recursive: true });
}
for (const log of result.logs) {
	console.error(log);
}
if (!result.success) {
	process.exitCode = 1;
}
