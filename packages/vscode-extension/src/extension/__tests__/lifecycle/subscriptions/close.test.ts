import { expect, it } from "../../runtime.ts";

import {
  subscriptionContext,
  subscriptionHarness,
  textDocumentCloseListeners,
} from "./support.ts";

for (const testCase of [
  {
    name: "supported file closes clear edited snapshots without touching diagnostics",
    scheme: "file",
    uri: "file:///package.json",
    supported: true,
    editedPresent: false,
  },
  {
    name: "non-file closes preserve dependency snapshots",
    scheme: "versionlens",
    uri: "versionlens:/schema.json",
    supported: true,
    editedPresent: true,
  },
  {
    name: "unsupported file closes preserve dependency snapshots",
    scheme: "file",
    uri: "file:///README.md",
    supported: false,
    editedPresent: true,
  },
] as const) {
  it(testCase.name, async (): Promise<void> => {
    const { registerExtensionSubscriptions } = await import(
      "../../../lifecycle/subscriptions.ts"
    );
    const uri = {
      scheme: testCase.scheme,
      toString: (): string => testCase.uri,
    };
    const document = { uri };
    const deletedUris: unknown[] = [];
    textDocumentCloseListeners.length = 0;
    subscriptionHarness.analyzeDocumentResult = {
      isSupportedManifest: testCase.supported,
    };
    const state = {
      snapshots: {
        editedDependencies: new Map([[testCase.uri, "edited"]]),
        savedDependencies: new Map([[testCase.uri, "saved"]]),
      },
      ui: {
        diagnostics: {
          delete(uriToDelete: unknown): void {
            deletedUris.push(uriToDelete);
          },
        },
        outputChannel: {},
      },
    };

    registerExtensionSubscriptions(
      state as never,
      subscriptionContext() as never,
    );
    textDocumentCloseListeners[0]?.(document);

    expect(state.snapshots.editedDependencies.has(testCase.uri)).toBe(
      testCase.editedPresent,
    );
    expect(state.snapshots.savedDependencies.get(testCase.uri)).toBe("saved");
    expect(deletedUris).toHaveLength(0);
  });
}
