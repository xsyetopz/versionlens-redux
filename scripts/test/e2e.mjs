#!/usr/bin/env bun
// Runs the extension in a real VS Code against the built server and a local
// fake npm registry, so it needs no network once VS Code is downloaded.

import { cp, mkdir, mkdtemp, readdir, realpath, rm } from "node:fs/promises";
import { createServer } from "node:http";
import { tmpdir } from "node:os";
import { join, resolve as resolvePath } from "node:path";
import process from "node:process";

import { runTests } from "@vscode/test-electron";

const root = resolvePath(import.meta.dir, "../..");
const extensionPath = join(root, "packages/vscode-extension");
const serverName =
	process.platform === "win32" ? "versionlens-lsp.exe" : "versionlens-lsp";
const serverPath =
	process.env["VERSIONLENS_LSP"] ?? join(root, "target/debug", serverName);

// Print VS Code's main, renderer and extension host logs so CI failures
// show the cause without downloading artifacts.
/** @param {string} directory */
const printLogs = async (directory) => {
	/** @type {string[]} */
	let entries;
	try {
		entries = await readdir(directory, { recursive: true });
	} catch {
		return;
	}
	const logs = entries.filter((name) =>
		/(^|\/)(main|renderer\d*|exthost\d*)\.log$/u.test(name),
	);
	const contents = await Promise.all(
		logs.map((name) => Bun.file(join(directory, name)).text()),
	);
	for (const [index, name] of logs.entries()) {
		console.error(`--- ${name} ---`);
		console.error(contents[index]);
	}
};

const packument = JSON.stringify({
	"dist-tags": { latest: "2.0.0" },
	versions: { "1.0.0": {}, "1.1.0": {}, "2.0.0": {} },
});
const registry = createServer((_request, response) => {
	response.setHeader("content-type", "application/json");
	response.end(packument);
});
/** @type {Promise<void>} */
const listening = new Promise((resolve) => {
	registry.listen(0, "127.0.0.1", () => {
		resolve();
	});
});
await listening;
const address = registry.address();
if (address === null || typeof address === "string") {
	throw new Error("the fake registry did not bind a TCP port");
}
const { port } = address;

// Resolved, because VS Code reports the real path of files and the server
// compares document and folder URIs as written.
const scratch = await realpath(
	await mkdtemp(join(tmpdir(), "versionlens-e2e-")),
);
const workspace = join(scratch, "workspace");
await cp(join(root, "tests/fixtures/vscode-e2e"), workspace, {
	recursive: true,
});
await mkdir(join(workspace, ".vscode"));
await Bun.write(
	join(workspace, ".vscode/settings.json"),
	JSON.stringify({
		"versionlens.npm.apiUrl": `http://127.0.0.1:${port}`,
		"versionlens.suggestions.showOnStartup": true,
		"versionlens.suggestions.showVulnerabilities": false,
	}),
);

// Bundle the test entry; VS Code loads it as CommonJS.
const built = await Bun.build({
	entrypoints: [join(root, "tests/vscode-extension/e2e.ts")],
	external: ["vscode"],
	format: "cjs",
	outdir: join(scratch, "tests"),
	target: "node",
});
if (built.success) {
	let code = 0;
	try {
		await runTests({
			cachePath: join(root, ".vscode-test"),
			extensionDevelopmentPath: extensionPath,
			extensionTestsEnv: { VERSIONLENS_LSP: serverPath },
			extensionTestsPath: join(scratch, "tests/e2e.js"),
			launchArgs: [
				workspace,
				"--disable-extensions",
				// Headless Linux CI: no GPU under Xvfb and no keyring on D-Bus.
				"--disable-gpu",
				"--disable-workspace-trust",
				"--use-inmemory-secretstorage",
				"--skip-welcome",
				"--skip-release-notes",
				"--logsPath",
				join(scratch, "logs"),
				"--user-data-dir",
				join(scratch, "user-data"),
			],
		});
	} catch (error) {
		console.error(error);
		code = 1;
	} finally {
		registry.close();
		registry.closeAllConnections();
		if (code === 0) {
			await rm(scratch, { force: true, recursive: true });
		} else {
			await printLogs(join(scratch, "logs"));
			console.error(`Logs are kept in ${scratch}`);
		}
	}
	process.exitCode = code;
} else {
	console.error(built.logs.map((log) => log.message).join("\n"));
	registry.close();
	registry.closeAllConnections();
	process.exitCode = 1;
}
