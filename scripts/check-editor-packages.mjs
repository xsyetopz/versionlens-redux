#!/usr/bin/env bun

import { createHash } from "node:crypto";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { arch, platform } from "node:process";

const linePattern = /\r?\n/u;
const version = JSON.parse(readFileSync("package.json", "utf8")).version;
const executableName =
	platform === "win32" ? "versionlens-lsp.exe" : "versionlens-lsp";
const vscodeArtifact = join(
	"packages",
	"vscode-extension",
	`versionlens-redux-${version}.vsix`,
);
const zedArtifact = join(
	"dist",
	`versionlens-redux-zed-extension-${platform}-${arch}.tar.gz`,
);
const jetbrainsArtifact = join(
	"packages",
	"jetbrains-plugin",
	"build",
	"distributions",
	`versionlens-jetbrains-plugin-${version}.zip`,
);

function run(command) {
	const result = Bun.spawnSync(command);
	if (result.exitCode !== 0) {
		process.stderr.write(result.stderr);
		process.exit(result.exitCode);
	}
	return result.stdout;
}

function requireEntry(entries, expected, artifact) {
	if (!entries.split(linePattern).includes(expected)) {
		throw new Error(`${artifact} does not contain ${expected}`);
	}
}

function digest(bytes) {
	return createHash("sha256").update(bytes).digest("hex");
}

function requireMatchingBinary(actual, expectedPath, artifact) {
	if (digest(actual) !== digest(readFileSync(expectedPath))) {
		throw new Error(`${artifact} contains a stale runtime binary`);
	}
}

const vscodeEntries = new TextDecoder().decode(
	run(["unzip", "-Z1", vscodeArtifact]),
);
requireEntry(
	vscodeEntries,
	"extension/native/versionlens_napi.node",
	vscodeArtifact,
);
requireMatchingBinary(
	run([
		"unzip",
		"-p",
		vscodeArtifact,
		"extension/native/versionlens_napi.node",
	]),
	join("packages", "vscode-extension", "native", "versionlens_napi.node"),
	vscodeArtifact,
);

const zedEntries = new TextDecoder().decode(run(["tar", "-tzf", zedArtifact]));
requireEntry(zedEntries, `bin/${executableName}`, zedArtifact);
requireMatchingBinary(
	run(["tar", "-xOzf", zedArtifact, `bin/${executableName}`]),
	join("target", "release", executableName),
	zedArtifact,
);

const temporaryDirectory = mkdtempSync(
	join(tmpdir(), "versionlens-jetbrains-"),
);
try {
	const outerEntries = new TextDecoder().decode(
		run(["unzip", "-Z1", jetbrainsArtifact]),
	);
	const pluginJar = outerEntries
		.split(linePattern)
		.find((entry) =>
			entry.endsWith(`/lib/versionlens-jetbrains-plugin-${version}.jar`),
		);
	if (!pluginJar) {
		throw new Error(`${jetbrainsArtifact} does not contain the plugin JAR`);
	}

	const jarPath = join(temporaryDirectory, "plugin.jar");
	writeFileSync(jarPath, run(["unzip", "-p", jetbrainsArtifact, pluginJar]));
	const jarEntries = new TextDecoder().decode(run(["unzip", "-Z1", jarPath]));
	requireEntry(jarEntries, `bin/${executableName}`, jetbrainsArtifact);
	requireMatchingBinary(
		run(["unzip", "-p", jarPath, `bin/${executableName}`]),
		join("target", "release", executableName),
		jetbrainsArtifact,
	);
} finally {
	rmSync(temporaryDirectory, { force: true, recursive: true });
}

console.log("Verified bundled runtimes in all three editor packages.");
