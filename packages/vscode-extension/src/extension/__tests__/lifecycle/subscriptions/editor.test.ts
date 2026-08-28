import { expect, it } from "../../runtime.ts";

import {
  activeEditorChangeListeners,
  createdWatcherPatterns,
  refreshedDocuments,
  subscriptionContext,
  subscriptionHarness,
  subscriptionState,
} from "./support.ts";

it("empty active editor changes update toolbar contexts without status UI", async (): Promise<void> => {
  const { registerExtensionSubscriptions } = await import(
    "../../../lifecycle/subscriptions.ts"
  );
  const context = subscriptionContext();
  activeEditorChangeListeners.length = 0;
  subscriptionHarness.updateContextCount = 0;
  subscriptionHarness.updateContextsResult = false;

  registerExtensionSubscriptions(
    subscriptionState() as never,
    context as never,
  );
  await activeEditorChangeListeners[0]?.(undefined);

  expect(subscriptionHarness.updateContextCount).toBe(1);
});

it("registers package file system watchers with extension subscriptions", async (): Promise<void> => {
  const { registerExtensionSubscriptions } = await import(
    "../../../lifecycle/subscriptions.ts"
  );
  const context = subscriptionContext();
  createdWatcherPatterns.length = 0;

  registerExtensionSubscriptions(
    subscriptionState() as never,
    context as never,
  );

  expect(
    createdWatcherPatterns.filter(
      (pattern): boolean =>
        (pattern as { pattern?: string }).pattern === "**/package.json",
    ),
  ).toHaveLength(1);
});

for (const testCase of [
  {
    name: "non-file active editor changes update contexts without refreshing diagnostics",
    scheme: "versionlens",
    uri: "versionlens:/schema.json",
    updateContextsResult: false,
  },
  {
    name: "unsupported file active editor changes update contexts without refreshing diagnostics",
    scheme: "file",
    uri: "file:///workspace/README.md",
    updateContextsResult: false,
  },
  {
    name: "supported workspace active editor changes update contexts without refreshing diagnostics",
    scheme: "file",
    uri: "file:///workspace/package.json",
    updateContextsResult: true,
  },
] as const) {
  it(testCase.name, async (): Promise<void> => {
    const { registerExtensionSubscriptions } = await import(
      "../../../lifecycle/subscriptions.ts"
    );
    const document = {
      uri: {
        scheme: testCase.scheme,
        toString: (): string => testCase.uri,
      },
    };
    activeEditorChangeListeners.length = 0;
    refreshedDocuments.length = 0;
    subscriptionHarness.updateContextCount = 0;
    subscriptionHarness.updateContextsResult = testCase.updateContextsResult;

    registerExtensionSubscriptions(
      subscriptionState() as never,
      subscriptionContext() as never,
    );
    await activeEditorChangeListeners[0]?.({ document });

    expect(subscriptionHarness.updateContextCount).toBe(1);
    expect(refreshedDocuments).toEqual([]);
  });
}
