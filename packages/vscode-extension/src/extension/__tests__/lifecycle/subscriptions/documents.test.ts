import { expect, it } from "../../runtime.ts";

import {
  refreshedDocuments,
  subscriptionContext,
  subscriptionHarness,
  subscriptionState,
  textDocumentChangeListeners,
} from "./support.ts";

const documentChangeCases = [
  {
    name: "active document edits refresh diagnostics and toolbar contexts",
    analyzeDocumentResult: { isSupportedManifest: true },
    contentChanges: [{ text: "changed" }],
    expectedRefreshes: 1,
    expectedUpdates: 1,
    uri: "file:///package.json",
  },
  {
    name: "unsupported text document changes do not refresh diagnostics",
    analyzeDocumentResult: { isSupportedManifest: false },
    contentChanges: [{ text: "changed" }],
    expectedRefreshes: 0,
    expectedUpdates: 0,
    uri: "file:///README.md",
  },
  {
    name: "empty text document changes without undo or redo do not refresh diagnostics",
    analyzeDocumentResult: { isSupportedManifest: true },
    contentChanges: [],
    expectedRefreshes: 0,
    expectedUpdates: 0,
    uri: "file:///package.json",
  },
] as const;

for (const testCase of documentChangeCases) {
  it(testCase.name, async (): Promise<void> => {
    const { registerExtensionSubscriptions } = await import(
      "../../../lifecycle/subscriptions.ts"
    );
    const document = { uri: { toString: (): string => testCase.uri } };
    textDocumentChangeListeners.length = 0;
    refreshedDocuments.length = 0;
    subscriptionHarness.updateContextCount = 0;
    subscriptionHarness.analyzeDocumentResult = testCase.analyzeDocumentResult;
    subscriptionHarness.activeTextEditor = { document };

    registerExtensionSubscriptions(
      subscriptionState() as never,
      subscriptionContext() as never,
    );
    await textDocumentChangeListeners[0]?.({
      contentChanges: [...testCase.contentChanges],
      document,
    });

    expect(refreshedDocuments).toHaveLength(testCase.expectedRefreshes);
    expect(subscriptionHarness.updateContextCount).toBe(
      testCase.expectedUpdates,
    );
  });
}

it("undo and redo text document changes refresh diagnostics without content changes", async (): Promise<void> => {
  const { registerExtensionSubscriptions } = await import(
    "../../../lifecycle/subscriptions.ts"
  );
  const document = { uri: { toString: (): string => "file:///package.json" } };
  const context = subscriptionContext();
  textDocumentChangeListeners.length = 0;
  refreshedDocuments.length = 0;
  subscriptionHarness.updateContextCount = 0;
  subscriptionHarness.activeTextEditor = { document };

  registerExtensionSubscriptions(
    subscriptionState() as never,
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
