#!/usr/bin/env bun
import { readdir, stat } from "node:fs/promises";
import { join } from "node:path";
import process from "node:process";

import { inherit } from "../lib/spawn.mjs";

const testRoots = [
	"packages/vscode-extension/src/extension/__tests__",
	"packages/vscode-extension/tests",
];

/**
 * @param {string} directory
 * @returns {Promise<string[]>}
 */
const collectTestFiles = async (directory) => {
	const names = await readdir(directory);
	const entries = names.toSorted();
	const nested = await Promise.all(
		entries.map(async (entry) => {
			const path = join(directory, entry);
			const info = await stat(path);
			if (info.isDirectory()) {
				return collectTestFiles(path);
			}
			return path.endsWith(".test.ts") ? [path] : [];
		}),
	);
	return nested.flat();
};

/**
 * @param {string} path
 * @returns {Promise<boolean>}
 */
const exists = async (path) => {
	try {
		await stat(path);
		return true;
	} catch {
		return false;
	}
};

/**
 * Run the files one after another, stopping at the first failure.
 * @param {string[]} files
 * @returns {Promise<number>} the exit code
 */
const runFiles = async ([file, ...rest]) => {
	if (file === undefined) {
		return 0;
	}
	const code = await inherit(["bun", "test", file]);
	return code === 0 ? runFiles(rest) : code;
};

// The server tests drive the real binary, so build it from the workspace.
const serverOverride = process.env["VERSIONLENS_LSP"];
if (serverOverride === undefined || serverOverride === "") {
	const code = await inherit([
		"cargo",
		"build",
		"-p",
		"versionlens-lsp",
		"--locked",
	]);
	if (code !== 0) {
		process.exitCode = code;
	}
}

if (process.exitCode === undefined) {
	const existing = await Promise.all(
		testRoots.map(async (testRoot) =>
			(await exists(testRoot)) ? collectTestFiles(testRoot) : [],
		),
	);
	process.exitCode = await runFiles(existing.flat());
}
