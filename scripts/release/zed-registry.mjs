#!/usr/bin/env bun
const registryPath = Bun.argv[2];
const version = Bun.argv[3];
if (
	registryPath === undefined ||
	registryPath === "" ||
	version === undefined ||
	version === ""
) {
	throw new Error(
		"usage: bun scripts/release/zed-registry.mjs <extensions.toml> <version>",
	);
}

const extensionId = "versionlens-lsp";
const entry = `[${extensionId}]\nsubmodule = "extensions/${extensionId}"\npath = "packages/zed-extension"\nversion = "${version}"\n`;
const source = await Bun.file(registryPath).text();
const sectionPattern = new RegExp(
	`^\\[${extensionId}\\]\\n(?:^(?!\\[).*(?:\\n|$))*`,
	"mu",
);
const next = sectionPattern.test(source)
	? source.replace(sectionPattern, entry)
	: `${source.trimEnd()}\n\n${entry}`;
await Bun.write(registryPath, next);
