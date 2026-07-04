#!/usr/bin/env bun

import { copyFileSync, mkdirSync } from "node:fs";
import { join } from "node:path";

const profile = Bun.argv.includes("--release") ? "release" : "debug";
const ext =
	process.platform === "win32"
		? "dll"
		: process.platform === "darwin"
			? "dylib"
			: "so";
const source = join(
	process.env.CARGO_TARGET_DIR ?? "target",
	profile,
	`libversionlens_napi.${ext}`,
);
const target = join(
	"packages",
	"vscode-extension",
	"native",
	"versionlens_napi.node",
);

mkdirSync(join("packages", "vscode-extension", "native"), { recursive: true });
copyFileSync(source, target);
