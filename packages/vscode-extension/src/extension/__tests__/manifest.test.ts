import { expect, test } from "bun:test";
import { resolve } from "node:path";

test("command activation events rely on VS Code 1.75 automatic generation", async () => {
	const local = await packageJson("packages/vscode-extension/package.json");
	const activationEvents = local.activationEvents ?? [];

	expect(activationEvents.some((event) => event.startsWith("onCommand:"))).toBe(
		false,
	);
});

async function packageJson(path: string) {
	return (await Bun.file(resolve(path)).json()) as {
		activationEvents?: string[];
		keywords?: string[];
		contributes?: {
			commands?: { command: string }[];
			jsonValidation?: unknown[];
			configuration?: {
				properties?: { [key: string]: unknown };
			};
		};
	};
}
