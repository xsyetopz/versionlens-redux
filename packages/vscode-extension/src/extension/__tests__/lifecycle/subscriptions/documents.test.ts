import { expect, it } from "../../runtime.ts";

import {
  refreshedDocuments,
  subscriptionHarness,
  textDocumentChangeListeners,
} from "./support.ts";

it("active document edits refresh diagnostics and toolbar contexts", async (): Promise<void> => {
  const { registerExtensionSubscriptions } = await import(
    "../../../lifecycle/subscriptions.ts"
  );
  const document = { uri: { toString: (): string => "file:///package.json" } };
  const context = { subscriptions: [] };
  textDocumentChangeListeners.length = 0;
  refreshedDocuments.length = 0;
  subscriptionHarness.updateContextCount = 0;
  subscriptionHarness.updateContextsResult = true;
  subscriptionHarness.analyzeDocumentResult = { isSupportedManifest: true };
  subscriptionHarness.activeTextEditor = { document };

  registerExtensionSubscriptions(
    {
      snapshots: {
        editedDependencies: new Map<string, string>(),
        savedDependencies: new Map<string, string>(),
      },
      ui: {
        diagnostics: { delete: (): undefined => undefined },
        outputChannel: {},
      },
    } as never,
    context as never,
  );
  await textDocumentChangeListeners[0]?.({
    contentChanges: [{ text: "changed" }],
    document,
  });

  expect(refreshedDocuments).toEqual([document]);
  expect(subscriptionHarness.updateContextCount).toBe(1);
});

it("unsupported text document changes do not refresh diagnostics", async (): Promise<void> => {
  const { registerExtensionSubscriptions } = await import(
    "../../../lifecycle/subscriptions.ts"
  );
  const document = { uri: { toString: (): string => "file:///README.md" } };
  const context = { subscriptions: [] };
  textDocumentChangeListeners.length = 0;
  refreshedDocuments.length = 0;
  subscriptionHarness.updateContextCount = 0;
  subscriptionHarness.analyzeDocumentResult = { isSupportedManifest: false };
  subscriptionHarness.activeTextEditor = { document };

  registerExtensionSubscriptions(
    {
      snapshots: {
        editedDependencies: new Map<string, string>(),
        savedDependencies: new Map<string, string>(),
      },
      ui: {
        diagnostics: { delete: (): undefined => undefined },
        outputChannel: {},
      },
    } as never,
    context as never,
  );
  await textDocumentChangeListeners[0]?.({
    contentChanges: [{ text: "changed" }],
    document,
  });

  expect(refreshedDocuments).toEqual([]);
  expect(subscriptionHarness.updateContextCount).toBe(0);
});

it("empty text document changes without undo or redo do not refresh diagnostics", async (): Promise<void> => {
  const { registerExtensionSubscriptions } = await import(
    "../../../lifecycle/subscriptions.ts"
  );
  const document = { uri: { toString: (): string => "file:///package.json" } };
  const context = { subscriptions: [] };
  textDocumentChangeListeners.length = 0;
  refreshedDocuments.length = 0;
  subscriptionHarness.updateContextCount = 0;
  subscriptionHarness.analyzeDocumentResult = { isSupportedManifest: true };
  subscriptionHarness.activeTextEditor = { document };

  registerExtensionSubscriptions(
    {
      snapshots: {
        editedDependencies: new Map<string, string>(),
        savedDependencies: new Map<string, string>(),
      },
      ui: {
        diagnostics: { delete: (): undefined => undefined },
        outputChannel: {},
      },
    } as never,
    context as never,
  );
  await textDocumentChangeListeners[0]?.({ contentChanges: [], document });

  expect(refreshedDocuments).toEqual([]);
  expect(subscriptionHarness.updateContextCount).toBe(0);
});

it("undo and redo text document changes refresh diagnostics without content changes", async (): Promise<void> => {
  const { registerExtensionSubscriptions } = await import(
    "../../../lifecycle/subscriptions.ts"
  );
  const document = { uri: { toString: (): string => "file:///package.json" } };
  const context = { subscriptions: [] };
  textDocumentChangeListeners.length = 0;
  refreshedDocuments.length = 0;
  subscriptionHarness.updateContextCount = 0;
  subscriptionHarness.activeTextEditor = { document };

  registerExtensionSubscriptions(
    {
      snapshots: {
        editedDependencies: new Map<string, string>(),
        savedDependencies: new Map<string, string>(),
      },
      ui: {
        diagnostics: { delete: (): undefined => undefined },
        outputChannel: {},
      },
    } as never,
    context as never,
  );
  await textDocumentChangeListeners[0]?.({
    contentChanges: [],
    document,
    reason: 1,
  });
  await textDocumentChangeListeners[0]?.({
    contentChanges: [],
    document,
    reason: 2,
  });

  expect(refreshedDocuments).toEqual([document, document]);
  expect(subscriptionHarness.updateContextCount).toBe(2);
});
