import { expect, test } from "bun:test";

import manifest from "../package.json";

const server = "../../../crates/versionlens-lsp/src";
const read = (path: string): Promise<string> =>
	Bun.file(new URL(path, import.meta.url)).text();
const matches = (text: string, pattern: RegExp): string[] =>
	[...text.matchAll(pattern)].map((match) => match[1] ?? "");

test("every server command is contributed, and every contributed command is handled", async () => {
	const serverCommands = matches(
		await read(`${server}/server.rs`),
		/^\s+"(versionlens\.\w+)",$/gmu
	);
	const local = matches(
		await read("./extension.ts"),
		/registerCommand\("(versionlens\.\w+)"/gu
	);
	const contributed = manifest.contributes.commands.map(
		({ command }) => command
	);
	expect(serverCommands.length).toBeGreaterThan(0);
	expect(new Set(contributed)).toEqual(new Set([...serverCommands, ...local]));
	expect(contributed).toHaveLength(serverCommands.length + local.length);
});

test("every setting the server reads is contributed", async () => {
	const keys = matches(
		await read(`${server}/config.rs`),
		/(?:flag|number|get)\("(\w+)"/gu
	).filter((key) => key !== "versionlens");
	const contributed = Object.keys(
		manifest.contributes.configuration.properties
	);
	expect(keys.length).toBeGreaterThan(5);
	for (const key of keys) {
		expect(contributed).toContain(`versionlens.${key}`);
	}
});
