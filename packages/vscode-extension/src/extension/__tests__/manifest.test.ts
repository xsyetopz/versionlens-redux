import { resolve } from "node:path";

import { expect, it } from "./runtime.ts";

interface PackageManifest {
  activationEvents?: string[];
  keywords?: string[];
  contributes?: {
    commands?: { command: string }[];
    jsonValidation?: unknown[];
    configuration?: { properties?: { [key: string]: unknown } };
  };
}

it("command activation events rely on VS Code 1.75 automatic generation", async (): Promise<void> => {
  const local = await packageJson("packages/vscode-extension/package.json");
  const activationEvents = local.activationEvents ?? [];

  expect(
    activationEvents.some((event): boolean => event.startsWith("onCommand:")),
  ).toBe(false);
});

it("all configured manifest languages activate the extension", async (): Promise<void> => {
  const local = await packageJson("packages/vscode-extension/package.json");
  const activationEvents = new Set(local.activationEvents ?? []);
  const { filePatternKeys } = await import("../config/keys/files.ts");

  for (const [, , languages] of filePatternKeys) {
    for (const language of languages) {
      expect(activationEvents).toContain(`onLanguage:${language}`);
    }
  }
});

it("CocoaPods cache and save settings use shared configuration contracts", async (): Promise<void> => {
  const local = await packageJson("packages/vscode-extension/package.json");
  const properties = local.contributes?.configuration?.properties ?? {};

  expect(properties["versionlens.cocoapods.caching.duration"]).toEqual(
    expect.objectContaining({
      type: "number",
      description: expect.stringContaining("minutes"),
    }),
  );
  expect(
    properties["versionlens.cocoapods.caching.duration"],
  ).not.toHaveProperty("default");
  expect(properties["versionlens.cocoapods.onSaveChanges"]).toEqual(
    expect.objectContaining({ type: "string", default: null }),
  );
});

async function packageJson(path: string): Promise<PackageManifest> {
  const packagePath = path;
  return (await Bun.file(resolve(packagePath)).json()) as PackageManifest;
}
