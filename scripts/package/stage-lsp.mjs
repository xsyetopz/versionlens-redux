#!/usr/bin/env bun
import { createHash } from "node:crypto";
import { chmod, mkdir, mkdtemp, readdir, rename, rm } from "node:fs/promises";
import { join } from "node:path";
import process from "node:process";

import { capture } from "../lib/spawn.mjs";
import { readTargets } from "./targets.mjs";

const targets = await readTargets();

/**
 * @param {string} vscodeTarget
 * @returns {string}
 */
const rustTargetFor = (vscodeTarget) => {
	const entry = targets.find(({ vscode }) => vscode === vscodeTarget);
	if (!entry) {
		throw new Error(`unsupported VS Code target: ${vscodeTarget}`);
	}
	return entry.rust;
};

/**
 * The `targets.json` entry of the machine's own Rust target.
 * @returns {Promise<import("./targets.mjs").PackageTarget>}
 */
const hostTarget = async () => {
	const result = await capture(["rustc", "-vV"]);
	const host = /^host:\s*(\S+)/mu.exec(result.stdout)?.[1];
	const entry = targets.find(({ rust }) => rust === host);
	if (!entry) {
		throw new Error(
			`Host Rust target ${host ?? "(unknown)"} is not listed in scripts/package/targets.json`,
		);
	}
	return entry;
};

/**
 * @param {string} rustTarget
 * @returns {string}
 */
const executableNameFor = (rustTarget) =>
	rustTarget.includes("windows") ? "versionlens-lsp.exe" : "versionlens-lsp";

/**
 * @param {string} archive
 * @param {string} destination
 * @returns {Promise<void>}
 */
const extract = async (archive, destination) => {
	let command = ["tar", "-xzf", archive, "-C", destination];
	if (archive.endsWith(".zip")) {
		command = ["unzip", "-o", "-q", archive, "-d", destination];
		if (process.platform === "win32") {
			const root = process.env["SystemRoot"] ?? String.raw`C:\Windows`;
			command = [
				join(root, "System32", "tar.exe"),
				"-xf",
				archive,
				"-C",
				destination,
			];
		}
	}
	const result = await capture(command);
	if (result.exitCode !== 0) {
		throw new Error(`Could not extract ${archive}: ${result.stderr}`);
	}
};

/**
 * @param {string} archivesDir
 * @param {string} archiveName
 * @returns {Promise<void>}
 */
const verifyChecksum = async (archivesDir, archiveName) => {
	const sums = Bun.file(join(archivesDir, "SHA256SUMS"));
	if (!(await sums.exists())) {
		return;
	}
	const text = await sums.text();
	const line = text
		.split("\n")
		.find((entry) => entry.trim().endsWith(` ${archiveName}`));
	if (line === undefined || line === "") {
		throw new Error(`SHA256SUMS has no entry for ${archiveName}`);
	}
	const expected = line.trim().split(/\s+/u)[0];
	const actual = createHash("sha256")
		.update(await Bun.file(join(archivesDir, archiveName)).bytes())
		.digest("hex");
	if (expected !== actual) {
		throw new Error(
			`${archiveName} checksum mismatch: expected ${expected}, got ${actual}`,
		);
	}
};

/**
 * Extract the build-lsp binary for `rustTarget` from `archivesDir` into
 * `destination` and return its path. Verifies against SHA256SUMS when present.
 * @param {string} archivesDir
 * @param {string} rustTarget
 * @param {string} destination
 * @returns {Promise<string>}
 */
const stageLspBinary = async (archivesDir, rustTarget, destination) => {
	const archives = await readdir(archivesDir);
	const archiveName = archives.find(
		(name) =>
			name.startsWith("versionlens-lsp-") &&
			(name.endsWith(`-${rustTarget}.tar.gz`) ||
				name.endsWith(`-${rustTarget}.zip`)),
	);
	if (archiveName === undefined || archiveName === "") {
		throw new Error(`No build-lsp archive for ${rustTarget} in ${archivesDir}`);
	}
	await verifyChecksum(archivesDir, archiveName);
	const executableName = executableNameFor(rustTarget);
	// Scratch lives under the destination so the rename never crosses volumes.
	await mkdir(destination, { recursive: true });
	const scratch = await mkdtemp(join(destination, ".extract-"));
	try {
		await extract(join(archivesDir, archiveName), scratch);
		const extracted = join(scratch, executableName);
		if (!(await Bun.file(extracted).exists())) {
			throw new Error(`${archiveName} does not contain ${executableName}`);
		}
		const staged = join(destination, executableName);
		await rm(staged, { force: true });
		await rename(extracted, staged);
		if (!rustTarget.includes("windows")) {
			await chmod(staged, 0o755);
		}
		return staged;
	} finally {
		await rm(scratch, { force: true, recursive: true });
	}
};

export { executableNameFor, hostTarget, rustTargetFor, stageLspBinary };

if (import.meta.main) {
	const [archivesDir, rustTarget, destination] = Bun.argv.slice(2);
	if (
		archivesDir === undefined ||
		archivesDir === "" ||
		rustTarget === undefined ||
		rustTarget === "" ||
		destination === undefined ||
		destination === ""
	) {
		throw new Error(
			"usage: bun scripts/package/stage-lsp.mjs <archives-dir> <rust-target> <destination-dir>",
		);
	}
	console.log(
		`Staged ${await stageLspBinary(archivesDir, rustTarget, destination)}`,
	);
}
