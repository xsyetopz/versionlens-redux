#!/usr/bin/env bun
// Compares cold starts of the original VersionLens 1.x (installed from the Marketplace) and this
// extension: each sample is a fresh VS Code instance and user profile on the same manifest, and
// the two alternate so network drift lands on both. VERSIONLENS_BENCH_RUNS sets the samples per
// side. `just vscode-bench` runs it in the Linux container, off screen.
import { mkdtemp, readdir, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import process from "node:process";

import { cli, instance } from "./vscode-instance.mjs";

const runs = Number(process.env["VERSIONLENS_BENCH_RUNS"] ?? "7");
const root = await mkdtemp(join(tmpdir(), "versionlens-bench-"));

const install = Bun.spawn(
	[
		cli,
		`--extensions-dir=${join(root, "original")}`,
		`--user-data-dir=${join(root, "install")}`,
		"--install-extension",
		"pflannery.vscode-versionlens"
	],
	{ stdout: "inherit", stderr: "inherit" }
);
if ((await install.exited) !== 0) {
	throw new Error("cannot install pflannery.vscode-versionlens");
}
const installed = await readdir(join(root, "original"));
const folder = installed.find((name) =>
	name.startsWith("pflannery.vscode-versionlens-")
);
if (folder === undefined) {
	throw new Error("the original extension did not install");
}
const sides = {
	original: join(root, "original", folder),
	rewrite: resolve("packages/vscode-extension")
};

const build = await Bun.build({
	entrypoints: ["packages/vscode-extension/src/live/bench.ts"],
	external: ["vscode"],
	format: "cjs",
	outdir: join(root, "tests"),
	target: "node"
});
if (!build.success) {
	throw new AggregateError(build.logs, "bench build failed");
}

/**
 * One cold start: a fresh profile and hidden instance, returning the milliseconds it printed.
 *
 * @param {"original" | "rewrite"} side
 * @param {number} index
 * @returns {Promise<number>}
 */
const sample = async (side, index) => {
	const run = join(root, `${side}-${index}`);
	const workspace = join(run, "workspace");
	await Bun.write(
		join(workspace, "requirements.txt"),
		"requests==2.31.0\nflask>=2.0\n"
	);
	// The original hides its lenses until toggled unless this is on; the rewrite reads it too.
	await Bun.write(
		join(workspace, ".vscode", "settings.json"),
		JSON.stringify({ "versionlens.suggestions.showOnStartup": true })
	);
	const { stdout: out } = await instance([
		`--extensionDevelopmentPath=${sides[side]}`,
		`--extensionTestsPath=${join(root, "tests", "bench.js")}`,
		`--user-data-dir=${join(run, "user")}`,
		`--extensions-dir=${join(run, "extensions")}`,
		"--disable-workspace-trust",
		"--skip-welcome",
		"--skip-release-notes",
		"--new-window",
		workspace
	]);
	const line = out.split("\n").find((text) => text.startsWith("BENCH "));
	if (line === undefined) {
		throw new Error(`${side} run ${index} printed no result:\n${out}`);
	}
	console.info(`${side} ${index}: ${line.slice(6)}`);
	return Number(line.split(" ")[1]);
};

/** @type {["original", "rewrite"]} */
const SIDES = ["original", "rewrite"];
/** @type {["rewrite", "original"]} */
const REVERSED = ["rewrite", "original"];
/** @type {Record<"original" | "rewrite", number[]>} */
const results = { original: [], rewrite: [] };
// Windows run one at a time, so no two samples share the machine or the network.
/** @param {number} index */
const round = async (index) => {
	if (index === runs) {
		return;
	}
	const [first, second] = index % 2 === 0 ? SIDES : REVERSED;
	results[first].push(await sample(first, index));
	results[second].push(await sample(second, index));
	await round(index + 1);
};
await round(0);
await rm(root, { force: true, recursive: true });
/** @param {number[]} values */
const median = (values) => {
	const sorted = values.toSorted((a, b) => a - b);
	return sorted[Math.floor(sorted.length / 2)];
};
console.info(`baseline: ${folder}, ${runs} cold starts per side`);
for (const [side, values] of Object.entries(results)) {
	console.info(
		`${side}: median ${median(values)} ms, min ${Math.min(...values)}, max ${Math.max(...values)} (${values.join(", ")})`
	);
}
