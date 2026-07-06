import { expect, test } from "bun:test";
import { resolve } from "node:path";

test("activation events preserve upstream manifest events", async () => {
	const upstream = await packageJson(
		"external/versionlens/vscode-versionlens/package.json",
	);
	const local = await packageJson("packages/vscode-extension/package.json");
	const upstreamEvents = upstream.activationEvents ?? [];

	expect(local.activationEvents?.slice(0, upstreamEvents.length)).toEqual(
		upstreamEvents,
	);
});

test("command activation events rely on VS Code 1.75 automatic generation", async () => {
	const local = await packageJson("packages/vscode-extension/package.json");
	const activationEvents = local.activationEvents ?? [];

	expect(activationEvents.some((event) => event.startsWith("onCommand:"))).toBe(
		false,
	);
});

test("package keywords match upstream manifest metadata", async () => {
	const upstream = await packageJson(
		"external/versionlens/vscode-versionlens/package.json",
	);
	const local = await packageJson("packages/vscode-extension/package.json");

	expect(local.keywords).toEqual(upstream.keywords);
});

test("jsonValidation contribution url matches upstream manifest", async () => {
	const upstream = await packageJson(
		"external/versionlens/vscode-versionlens/package.json",
	);
	const local = await packageJson("packages/vscode-extension/package.json");

	expect(local.contributes?.jsonValidation).toEqual(
		upstream.contributes?.jsonValidation,
	);
});

test("common configuration contribution schemas match upstream manifest", async () => {
	const upstream = await packageJson(
		"external/versionlens/vscode-versionlens/package.json",
	);
	const local = await packageJson("packages/vscode-extension/package.json");
	const upstreamProperties =
		upstream.contributes?.configuration?.properties ?? {};
	const localProperties = local.contributes?.configuration?.properties ?? {};

	for (const key of Object.keys(upstreamProperties)) {
		if (key.endsWith(".files") || key.endsWith(".dependencyProperties")) {
			expect(omitDefault(localProperties[key])).toEqual(
				omitDefault(upstreamProperties[key]),
			);
			continue;
		}
		expect(localProperties[key]).toEqual(upstreamProperties[key]);
	}
});

test("enabledProviders contribution schema matches upstream", async () => {
	const upstream = await packageJson(
		"external/versionlens/vscode-versionlens/package.json",
	);
	const local = await packageJson("packages/vscode-extension/package.json");

	expect(
		local.contributes?.configuration?.properties?.[
			"versionlens.enabledProviders"
		],
	).toEqual(
		upstream.contributes?.configuration?.properties?.[
			"versionlens.enabledProviders"
		],
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

function omitDefault(value: unknown) {
	if (value === null || typeof value !== "object" || Array.isArray(value)) {
		return value;
	}
	const { default: _default, ...rest } = value as {
		default?: unknown;
		[key: string]: unknown;
	};
	return rest;
}
