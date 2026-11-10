import { resolve } from "node:path";

import { expect, it } from "./runtime.ts";

interface PropertySchema {
	default?: unknown;
	description?: string;
	items?: { enum?: string[] };
	type?: string;
}

interface PackageManifest {
	activationEvents: string[];
	contributes: {
		configuration: { properties: Record<string, PropertySchema> };
	};
}

const isPackageManifest = (value: unknown): value is PackageManifest =>
	typeof value === "object" &&
	value !== null &&
	"activationEvents" in value &&
	Array.isArray(value.activationEvents) &&
	"contributes" in value &&
	typeof value.contributes === "object";

const packageJson = async (path: string): Promise<PackageManifest> => {
	const manifest: unknown = await Bun.file(resolve(path)).json();
	if (!isPackageManifest(manifest)) {
		throw new TypeError(`Unexpected package manifest shape in ${path}`);
	}
	return manifest;
};

const packagePath = "packages/vscode-extension/package.json";

it("command activation events rely on VS Code 1.75 automatic generation", async (): Promise<void> => {
	const local = await packageJson(packagePath);

	expect(
		local.activationEvents.some((event): boolean =>
			event.startsWith("onCommand:"),
		),
	).toBe(false);
});

it("all configured manifest languages activate the extension", async (): Promise<void> => {
	const local = await packageJson(packagePath);
	const activationEvents = new Set(local.activationEvents);
	const { filePatternKeys } = await import("../config/keys.ts");

	for (const entry of filePatternKeys) {
		for (const language of entry[2]) {
			expect(activationEvents).toContain(`onLanguage:${language}`);
		}
	}
});

it("CocoaPods cache and save settings use shared configuration contracts", async (): Promise<void> => {
	const local = await packageJson(packagePath);
	const { properties } = local.contributes.configuration;
	const duration = properties["versionlens.cocoapods.caching.duration"];

	expect(duration?.type).toBe("number");
	expect(duration?.description).toContain("minutes");
	expect(duration).not.toHaveProperty("default");
	expect(properties["versionlens.cocoapods.onSaveChanges"]).toMatchObject({
		type: "string",
		default: null,
	});
});

it("enabledProviders offers every provider the client can select", async (): Promise<void> => {
	const local = await packageJson(packagePath);
	const { properties } = local.contributes.configuration;
	const { filePatternKeys } = await import("../config/keys.ts");

	expect(properties["versionlens.enabledProviders"]?.items?.enum).toEqual(
		filePatternKeys.map(([provider]): string => provider),
	);
});
