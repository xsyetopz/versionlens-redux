#!/usr/bin/env bun
import { chmod, copyFile, mkdir, rm } from "node:fs/promises";
import { join } from "node:path";
import {
	arch as hostArchitecture,
	platform as hostPlatform,
} from "node:process";

import { capture } from "../lib/spawn.mjs";
import { readTargets } from "./targets.mjs";

const targets = await readTargets();
/** @type {Map<string, string>} */
const nativeEditorTargets = new Map(
	targets.map(({ vscode, rust }) => [vscode, rust]),
);

/**
 * @param {string[]} args
 * @param {string} packageName
 * @returns {void}
 */
const assertNativeRequest = (args, packageName) => {
	const [requestedPlatform, requestedArchitecture] = args;
	if (args.length === 0) {
		return;
	}
	if (args.length !== 3) {
		throw new Error(
			`${packageName} target packaging requires platform, architecture, and Rust target arguments.`,
		);
	}
	if (
		requestedPlatform !== hostPlatform ||
		requestedArchitecture !== hostArchitecture
	) {
		throw new Error(
			`Cannot package ${requestedPlatform}-${requestedArchitecture} on ${hostPlatform}-${hostArchitecture}; use a native runner.`,
		);
	}
};

/**
 * @param {string[]} args
 * @param {string} packageName
 */
const resolveNativeEditorTarget = (args, packageName) => {
	assertNativeRequest(args, packageName);
	const [requestedPlatform, requestedArchitecture, requestedRustTarget] = args;

	const platform = requestedPlatform ?? hostPlatform;
	const architecture = requestedArchitecture ?? hostArchitecture;
	const editorTarget = `${platform}-${architecture}`;
	const expectedRustTarget = nativeEditorTargets.get(editorTarget);
	if (expectedRustTarget === undefined || expectedRustTarget === "") {
		throw new Error(
			`Unsupported ${packageName} package target: ${editorTarget}`,
		);
	}
	if (
		requestedRustTarget !== undefined &&
		requestedRustTarget !== "" &&
		requestedRustTarget !== expectedRustTarget
	) {
		throw new Error(
			`${editorTarget} requires Rust target ${expectedRustTarget}, received ${requestedRustTarget}.`,
		);
	}

	const executableName =
		platform === "win32" ? "versionlens-lsp.exe" : "versionlens-lsp";
	const sourceParts = ["target"];
	if (requestedRustTarget !== undefined && requestedRustTarget !== "") {
		sourceParts.push(requestedRustTarget);
	}
	sourceParts.push("release", executableName);

	return {
		architecture,
		editorTarget,
		executableName,
		platform,
		rustTarget: requestedRustTarget,
		sourceParts,
	};
};

const { editorTarget, executableName, platform, sourceParts } =
	resolveNativeEditorTarget(Bun.argv.slice(2), "Neovim");
const packageRoot = join("packages", "neovim-plugin");
const binaryDirectory = join(packageRoot, "bin", editorTarget);
const bundledBinary = join(binaryDirectory, executableName);
const output = join(
	"dist",
	`versionlens-redux-neovim-plugin-${editorTarget}.tar.gz`,
);

await mkdir(binaryDirectory, { recursive: true });
await mkdir("dist", { recursive: true });
try {
	await copyFile(join(...sourceParts), bundledBinary);
	if (platform !== "win32") {
		await chmod(bundledBinary, 0o755);
	}
	const result = await capture([
		"tar",
		"-czf",
		output,
		"-C",
		packageRoot,
		"README.md",
		"LICENSE",
		"doc",
		"lua",
		"plugin",
		"bin",
	]);
	if (result.exitCode !== 0) {
		throw new Error(`Could not create ${output}: ${result.stderr}`);
	}
} finally {
	await rm(join(packageRoot, "bin"), { force: true, recursive: true });
}

console.log(`Packaged ${output}`);
