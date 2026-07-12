#!/usr/bin/env bun

import { readFileSync } from "node:fs";

const vscodeTarget = Bun.argv[2];
const rustTarget = Bun.argv[3];
if (!(vscodeTarget && rustTarget)) {
	throw new Error(
		"usage: bun scripts/package-vscode-target.mjs <vscode-target> <rust-target>",
	);
}
const targetTriples = new Map([
	["win32-x64", "x86_64-pc-windows-msvc"],
	["win32-arm64", "aarch64-pc-windows-msvc"],
	["linux-x64", "x86_64-unknown-linux-gnu"],
	["linux-arm64", "aarch64-unknown-linux-gnu"],
	["linux-armhf", "armv7-unknown-linux-gnueabihf"],
	["alpine-x64", "x86_64-unknown-linux-musl"],
	["alpine-arm64", "aarch64-unknown-linux-musl"],
	["darwin-x64", "x86_64-apple-darwin"],
	["darwin-arm64", "aarch64-apple-darwin"],
]);
const expectedRustTarget = targetTriples.get(vscodeTarget);
if (!expectedRustTarget) {
	throw new Error(`unsupported VS Code target: ${vscodeTarget}`);
}
if (rustTarget !== expectedRustTarget) {
	throw new Error(
		`VS Code target ${vscodeTarget} requires Rust target ${expectedRustTarget}, not ${rustTarget}`,
	);
}
function run(command, options = {}) {
	const result = Bun.spawnSync(command, {
		...options,
		stdout: "inherit",
		stderr: "inherit",
	});
	if (result.exitCode !== 0) process.exit(result.exitCode);
}
run(["bun", "scripts/build-native.mjs", "--release", "--target", rustTarget]);
run(["bun", "scripts/build-extension.mjs"]);
const version = JSON.parse(readFileSync("package.json", "utf8")).version;
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
console.log(`Packaged packages/vscode-extension/${output}`);
