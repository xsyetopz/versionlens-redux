import { expect, it } from "../../runtime.ts";

import { subscriptionHarness, textDocumentCloseListeners } from "./support.ts";

it("supported file closes clear edited snapshots without touching diagnostics", async (): Promise<void> => {
  const { registerExtensionSubscriptions } = await import(
    "../../../lifecycle/subscriptions.ts"
  );
  const uri = {
    scheme: "file",
    toString: (): string => "file:///package.json",
  };
  const document = { uri };
  const context = { subscriptions: [] };
  const deletedUris: unknown[] = [];
  textDocumentCloseListeners.length = 0;
  subscriptionHarness.analyzeDocumentResult = { isSupportedManifest: true };

  const state = {
    snapshots: {
      editedDependencies: new Map([[uri.toString(), "edited"]]),
      savedDependencies: new Map([[uri.toString(), "saved"]]),
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

  registerExtensionSubscriptions(state as never, context as never);
  textDocumentCloseListeners[0]?.(document);

  expect(state.snapshots.editedDependencies.has(uri.toString())).toBe(false);
  expect(state.snapshots.savedDependencies.get(uri.toString())).toBe("saved");
  expect(deletedUris).toEqual([]);
});

it("non-file closes preserve dependency snapshots", async (): Promise<void> => {
  const { registerExtensionSubscriptions } = await import(
    "../../../lifecycle/subscriptions.ts"
  );
  const uri = {
    scheme: "versionlens",
    toString: (): string => "versionlens:/schema.json",
  };
  const document = { uri };
  const context = { subscriptions: [] };
  textDocumentCloseListeners.length = 0;
  subscriptionHarness.analyzeDocumentResult = { isSupportedManifest: true };

  const state = {
    snapshots: {
      editedDependencies: new Map([[uri.toString(), "edited"]]),
      savedDependencies: new Map([[uri.toString(), "saved"]]),
    },
    ui: {
      diagnostics: { delete: (): undefined => undefined },
      outputChannel: {},
    },
  };

  registerExtensionSubscriptions(state as never, context as never);
  textDocumentCloseListeners[0]?.(document);

  expect(state.snapshots.editedDependencies.get(uri.toString())).toBe("edited");
  expect(state.snapshots.savedDependencies.get(uri.toString())).toBe("saved");
});

it("unsupported file closes preserve dependency snapshots", async (): Promise<void> => {
  const { registerExtensionSubscriptions } = await import(
    "../../../lifecycle/subscriptions.ts"
  );
  const uri = { scheme: "file", toString: (): string => "file:///README.md" };
  const document = { uri };
  const context = { subscriptions: [] };
  textDocumentCloseListeners.length = 0;
  subscriptionHarness.analyzeDocumentResult = { isSupportedManifest: false };

  const state = {
    snapshots: {
      editedDependencies: new Map([[uri.toString(), "edited"]]),
      savedDependencies: new Map([[uri.toString(), "saved"]]),
    },
    ui: {
      diagnostics: { delete: (): undefined => undefined },
      outputChannel: {},
    },
  };

  registerExtensionSubscriptions(state as never, context as never);
  textDocumentCloseListeners[0]?.(document);

  expect(state.snapshots.editedDependencies.get(uri.toString())).toBe("edited");
  expect(state.snapshots.savedDependencies.get(uri.toString())).toBe("saved");
});
