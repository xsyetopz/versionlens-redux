#!/usr/bin/env bun
// Builds every editor package for the host: the VS Code VSIX, the Zed extension, the Neovim
// plugin, and the JetBrains plugin zip with the host's server staged under bin/<rust-target>/.
import { copyFile, mkdir, mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { capture, inherit } from "../lib/spawn.mjs";
import { readTargets } from "./targets.mjs";

const HOST_PATTERN = /^host:\s*(?<triple>\S+)/mu;

/**
 * @param {string[]} command
 * @returns {Promise<void>}
 */
const run = async (command) => {
	console.log(`$ ${command.join(" ")}`);
	const exitCode = await inherit(command);
	if (exitCode !== 0) {
		throw new Error(`exit code ${exitCode}: ${command.join(" ")}`);
	}
};

/**
 * Copies the release server into root/<host-triple>/, the layout the JetBrains build bundles.
 * @param {string} root
 * @returns {Promise<void>}
 */
const stageHostBinary = async (root) => {
	const { stdout } = await capture(["rustc", "-vV"]);
	const host = HOST_PATTERN.exec(stdout)?.groups?.["triple"];
	const targets = await readTargets();
	if (host === undefined || !targets.some(({ rust }) => rust === host)) {
		throw new Error(
			`host target ${host ?? "(unknown)"} is not in scripts/package/targets.json`
		);
	}
	const executable = host.includes("windows")
		? "versionlens-lsp.exe"
		: "versionlens-lsp";
	await mkdir(join(root, host), { recursive: true });
	await copyFile(
		join("target", "release", executable),
		join(root, host, executable)
	);
};

const binaries = await mkdtemp(join(tmpdir(), "versionlens-lsp-"));
try {
	await run(["bun", "run", "package"]);
	await run([
		"cargo",
		"build",
		"-p",
		"versionlens-lsp",
		"--release",
		"--locked"
	]);
	await stageHostBinary(binaries);
	await run([
		"cargo",
		"build",
		"--manifest-path",
		"packages/zed-extension/Cargo.toml",
		"--release",
		"--locked"
	]);
	await run(["bun", "scripts/package/neovim.mjs"]);
	await run([
		"just",
		"jvm::gradle",
		":jetbrains-plugin:buildPlugin",
		`-PversionlensBinariesDir=${binaries}`
	]);
} finally {
	await rm(binaries, { force: true, recursive: true });
}
