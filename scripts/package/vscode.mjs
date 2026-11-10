#!/usr/bin/env bun
import { copyFileSync, mkdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import process from "node:process";

import {
	executableNameFor,
	hostTarget,
	rustTargetFor,
	stageLspBinary,
} from "./stage-lsp.mjs";

function run(command, options = {}) {
	const result = Bun.spawnSync(command, {
		...options,
		stdout: "inherit",
		stderr: "inherit",
	});
	if (result.exitCode !== 0) {
		process.exit(result.exitCode);
	}
}

// A VSIX without a target installs next to a targeted one of the same
// version instead of replacing it, so every VSIX names its target. With no
// arguments, the server is built for this machine.
const [, , requestedTarget, archivesDir] = Bun.argv;
if (Boolean(requestedTarget) !== Boolean(archivesDir)) {
	throw new Error(
		"usage: bun scripts/package/vscode.mjs [<vscode-target> <build-lsp-archives-dir>]",
	);
}
const serverDirectory = join("packages", "vscode-extension", "server");
let vscodeTarget = requestedTarget;
let server;
if (archivesDir) {
	server = stageLspBinary(
		archivesDir,
		rustTargetFor(vscodeTarget),
		serverDirectory,
	);
} else {
	const host = hostTarget();
	vscodeTarget = host.vscode;
	run(["cargo", "build", "-p", "versionlens-lsp", "--release", "--locked"]);
	const executable = executableNameFor(host.rust);
	mkdirSync(serverDirectory, { recursive: true });
	server = join(serverDirectory, executable);
	copyFileSync(join("target", "release", executable), server);
}
run(["bun", "scripts/package/extension.mjs"]);
const { version } = JSON.parse(readFileSync("package.json", "utf8"));
const output = `versionlens-redux-${version}-${vscodeTarget}.vsix`;
run(
	[
		"bunx",
		"vsce",
		"package",
		"--no-dependencies",
		"--target",
		vscodeTarget,
		"--out",
		output,
	],
	{ cwd: "packages/vscode-extension" },
);
run([
	"bun",
	"scripts/package/vsix.mjs",
	`packages/vscode-extension/${output}`,
	server,
	vscodeTarget,
]);
console.log(`Packaged packages/vscode-extension/${output}`);
