#!/usr/bin/env bun

import { copyFileSync, mkdirSync } from "node:fs";
import { join } from "node:path";

const profile = Bun.argv.includes("--release") ? "release" : "debug";
const targetIndex = Bun.argv.indexOf("--target");
const rustTarget = targetIndex === -1 ? undefined : Bun.argv[targetIndex + 1];
if (targetIndex !== -1 && !rustTarget) {
	throw new Error("--target requires a Rust target triple");
}
const ext = rustTarget?.includes("windows")
	? "dll"
	: rustTarget?.includes("darwin")
		? "dylib"
		: rustTarget
			? "so"
			: process.platform === "win32"
				? "dll"
				: process.platform === "darwin"
					? "dylib"
					: "so";
const source = join(
	process.env.CARGO_TARGET_DIR ?? "target",
	...(rustTarget ? [rustTarget] : []),
	profile,
	rustTarget?.includes("windows")
		? "versionlens_napi.dll"
		: `libversionlens_napi.${ext}`,
);
const target = join(
	"packages",
	"vscode-extension",
	"native",
	"versionlens_napi.node",
);

mkdirSync(join("packages", "vscode-extension", "native"), { recursive: true });
copyFileSync(source, target);
