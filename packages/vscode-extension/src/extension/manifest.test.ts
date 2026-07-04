import { expect, test } from "bun:test";
import { resolve } from "node:path";

test("activation events match upstream manifest", async () => {
	const upstream = await packageJson(
		"external/versionlens/vscode-versionlens/package.json",
	);
	const local = await packageJson("packages/vscode-extension/package.json");

	expect(local.activationEvents).toEqual(upstream.activationEvents);
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
			jsonValidation?: unknown[];
			configuration?: {
				properties?: { [key: string]: unknown };
			};
		};
	};
}
