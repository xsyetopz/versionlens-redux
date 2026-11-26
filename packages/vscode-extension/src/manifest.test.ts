import { expect, test } from "bun:test";

import manifest from "../package.json";
import { LEVEL_COMMANDS } from "./levels.ts";

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
	const local = [
		...matches(
			await read("./commands.ts"),
			/registerCommand\("(versionlens\.\w+)"/gu
		),
		...LEVEL_COMMANDS.map(({ id }) => id)
	];
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
		/(?:flag|number|strings|list|get)\((?:section, )?"(\w+)"/gu
	).filter((key) => key !== "versionlens");
	const contributed = Object.keys(
		manifest.contributes.configuration.properties
	);
	expect(keys.length).toBeGreaterThan(5);
	for (const key of keys) {
		expect(contributed).toContain(`versionlens.${key}`);
	}
});

test("every editor title and palette entry names a contributed command", () => {
	const contributed = new Set(
		manifest.contributes.commands.map(({ command }) => command)
	);
	const menus = Object.values(manifest.contributes.menus).flat();
	for (const { command } of menus) {
		expect(contributed).toContain(command);
	}
});

test("every icon a command names ships with the extension", async () => {
	const icons = manifest.contributes.commands.flatMap((command) => {
		const icon: unknown = Reflect.get(command, "icon");
		if (typeof icon === "string") {
			return [icon];
		}
		return typeof icon === "object" && icon !== null
			? Object.values(icon).map(String)
			: [];
	});
	expect(icons.length).toBeGreaterThan(5);
	const exists = await Promise.all(
		icons.map((icon) =>
			Bun.file(new URL(`../${icon}`, import.meta.url)).exists()
		)
	);
	expect(exists.every(Boolean)).toBe(true);
});
