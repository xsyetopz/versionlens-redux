#!/usr/bin/env bun
import { copyFile, mkdir } from "node:fs/promises";
import { join } from "node:path";
import process from "node:process";

import { manifestVersion, parseJson } from "../lib/json.mjs";
import { inherit } from "../lib/spawn.mjs";
import {
	executableNameFor,
	hostTarget,
	rustTargetFor,
	stageLspBinary,
} from "./stage-lsp.mjs";

/**
 * Run a step; a failing one ends the packaging with its exit code.
 * @param {string[]} command
 * @param {{ cwd?: string }} [options]
 * @returns {Promise<boolean>} whether the step succeeded
 */
const run = async (command, options = {}) => {
	const exitCode = await inherit(command, options);
	if (exitCode !== 0) {
		process.exitCode = exitCode;
	}
	return exitCode === 0;
};

/**
 * @param {string} requestedTarget
 * @param {string} archivesDir
 * @param {string} serverDirectory
 * @returns {Promise<{ vscodeTarget: string, server: string } | null>}
 */
const stageServer = async (requestedTarget, archivesDir, serverDirectory) => {
	if (requestedTarget !== "" && archivesDir !== "") {
		return {
			server: await stageLspBinary(
				archivesDir,
				rustTargetFor(requestedTarget),
				serverDirectory,
			),
			vscodeTarget: requestedTarget,
		};
	}
	const host = await hostTarget();
	if (
		!(await run([
			"cargo",
			"build",
			"-p",
			"versionlens-lsp",
			"--release",
			"--locked",
		]))
	) {
		return null;
	}
	const executable = executableNameFor(host.rust);
	await mkdir(serverDirectory, { recursive: true });
	const server = join(serverDirectory, executable);
	await copyFile(join("target", "release", executable), server);
	return { server, vscodeTarget: host.vscode };
};

/**
 * @param {string} server
 * @param {string} vscodeTarget
 * @returns {Promise<void>}
 */
const packageExtension = async (server, vscodeTarget) => {
	if (!(await run(["bun", "scripts/package/extension.mjs"]))) {
		return;
	}
	const version = manifestVersion(
		parseJson(await Bun.file("package.json").text()),
	);
	const output = `versionlens-redux-${version}-${vscodeTarget}.vsix`;
	const packaged = await run(
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
	if (!packaged) {
		return;
	}
	const verified = await run([
		"bun",
		"scripts/package/vsix.mjs",
		`packages/vscode-extension/${output}`,
		server,
		vscodeTarget,
	]);
	if (!verified) {
		return;
	}
	console.log(`Packaged packages/vscode-extension/${output}`);
};

// A VSIX without a target installs next to a targeted one of the same
// version instead of replacing it, so every VSIX names its target. With no
// arguments, the server is built for this machine.
const requestedTarget = Bun.argv[2] ?? "";
const archivesDir = Bun.argv[3] ?? "";
if ((requestedTarget !== "") !== (archivesDir !== "")) {
	throw new Error(
		"usage: bun scripts/package/vscode.mjs [<vscode-target> <build-lsp-archives-dir>]",
	);
}
const staged = await stageServer(
	requestedTarget,
	archivesDir,
	join("packages", "vscode-extension", "server"),
);
if (staged !== null) {
	await packageExtension(staged.server, staged.vscodeTarget);
}
