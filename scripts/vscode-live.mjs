#!/usr/bin/env bun
// Runs packages/vscode-extension/src/live/run.ts in a real, isolated VS Code window against the
// live registries. VSCODE_BIN overrides the macOS application path.
import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import process from "node:process";

const code =
	process.env["VSCODE_BIN"] ??
	"/Applications/Visual Studio Code.app/Contents/MacOS/Code";
const root = await mkdtemp(join(tmpdir(), "versionlens-live-"));
const workspace = join(root, "workspace");
await Bun.write(
	join(workspace, "requirements.txt"),
	"requests==2.31.0\nflask>=2.0\n"
);
await Bun.write(
	join(workspace, "libraries.hxml"),
	"-lib heaps:1.10.0\n-lib format\n"
);
await Bun.write(join(workspace, "dockerfile"), "FROM alpine:3.18\n");
await Bun.write(join(workspace, "main.ts"), "export const x = 1;\n");
const build = await Bun.build({
	entrypoints: ["packages/vscode-extension/src/live/run.ts"],
	external: ["vscode"],
	format: "cjs",
	outdir: join(root, "tests"),
	target: "node"
});
if (!build.success) {
	throw new AggregateError(build.logs, "live test build failed");
}
const logs = join(root, "logs");
const child = Bun.spawn(
	[
		code,
		`--extensionDevelopmentPath=${resolve("packages/vscode-extension")}`,
		`--extensionTestsPath=${join(root, "tests", "run.js")}`,
		`--user-data-dir=${join(root, "user")}`,
		`--extensions-dir=${join(root, "extensions")}`,
		`--logsPath=${logs}`,
		"--disable-workspace-trust",
		"--skip-welcome",
		"--skip-release-notes",
		"--new-window",
		workspace
	],
	{
		env: { ...process.env, VERSIONLENS_LIVE_LOGS: logs },
		stderr: "ignore",
		stdout: "inherit"
	}
);
const exitCode = await child.exited;
await rm(root, { force: true, recursive: true });
process.exitCode = exitCode;
