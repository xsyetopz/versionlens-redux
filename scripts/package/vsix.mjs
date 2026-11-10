#!/usr/bin/env bun
import { createHash } from "node:crypto";
import { basename } from "node:path";

const vsixPath = Bun.argv[2];
const serverPath = Bun.argv[3];
const target = Bun.argv[4];
if (
	vsixPath === undefined ||
	vsixPath === "" ||
	serverPath === undefined ||
	serverPath === "" ||
	target === undefined ||
	target === ""
) {
	throw new Error(
		"usage: bun scripts/package/vsix.mjs <vsix> <server-binary> <vscode-target>",
	);
}

/**
 * @param {Uint8Array} buffer
 * @returns {string}
 */
const digest = (buffer) => createHash("sha256").update(buffer).digest("hex");

/**
 * @param {string} entry
 * @returns {Promise<Uint8Array>}
 */
const unzipEntry = async (entry) => {
	const child = Bun.spawn(["unzip", "-p", vsixPath, entry], {
		stderr: "inherit",
		stdout: "pipe",
	});
	const [output, exitCode] = await Promise.all([
		new Response(child.stdout).bytes(),
		child.exited,
	]);
	if (exitCode !== 0) {
		throw new Error(`unzip -p ${vsixPath} ${entry} exited with ${exitCode}`);
	}
	return output;
};

const serverEntry = `extension/server/${basename(serverPath)}`;
const packagedServer = await unzipEntry(serverEntry);
if (digest(packagedServer) !== digest(await Bun.file(serverPath).bytes())) {
	throw new Error(`${vsixPath} ${serverEntry} does not match ${serverPath}`);
}
const manifest = new TextDecoder().decode(
	await unzipEntry("extension.vsixmanifest"),
);
if (!manifest.includes(`TargetPlatform="${target}"`)) {
	throw new Error(`${vsixPath} target is not ${target}`);
}
