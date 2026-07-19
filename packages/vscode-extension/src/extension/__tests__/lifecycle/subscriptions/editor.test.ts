import { expect, it } from "../../runtime.ts";

import {
  activeEditorChangeListeners,
  createdWatcherPatterns,
  refreshedDocuments,
  subscriptionHarness,
} from "./support.ts";

it("empty active editor changes update toolbar contexts without status UI", async (): Promise<void> => {
  const { registerExtensionSubscriptions } = await import(
    "../../../lifecycle/subscriptions.ts"
  );
  const context = { subscriptions: [] };
  activeEditorChangeListeners.length = 0;
  subscriptionHarness.updateContextCount = 0;
  subscriptionHarness.updateContextsResult = false;

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
  await activeEditorChangeListeners[0]?.(undefined);

  expect(subscriptionHarness.updateContextCount).toBe(1);
});

it("registers package file system watchers with extension subscriptions", async (): Promise<void> => {
  const { registerExtensionSubscriptions } = await import(
    "../../../lifecycle/subscriptions.ts"
  );
  const context = { subscriptions: [] };
  createdWatcherPatterns.length = 0;

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

  expect(
    createdWatcherPatterns.filter(
      (pattern): boolean =>
        (pattern as { pattern?: string }).pattern === "**/package.json",
    ),
  ).toHaveLength(1);
});

it("non-file active editor changes update contexts without refreshing diagnostics", async (): Promise<void> => {
  const { registerExtensionSubscriptions } = await import(
    "../../../lifecycle/subscriptions.ts"
  );
  const context = { subscriptions: [] };
  const document = {
    uri: {
      scheme: "versionlens",
      toString: (): string => "versionlens:/schema.json",
    },
  };
  activeEditorChangeListeners.length = 0;
  refreshedDocuments.length = 0;
  subscriptionHarness.updateContextCount = 0;
  subscriptionHarness.updateContextsResult = false;

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
  await activeEditorChangeListeners[0]?.({ document });

  expect(subscriptionHarness.updateContextCount).toBe(1);
  expect(refreshedDocuments).toEqual([]);
});

it("unsupported file active editor changes update contexts without refreshing diagnostics", async (): Promise<void> => {
  const { registerExtensionSubscriptions } = await import(
    "../../../lifecycle/subscriptions.ts"
  );
  const context = { subscriptions: [] };
  const document = {
    uri: {
      scheme: "file",
      toString: (): string => "file:///workspace/README.md",
    },
  };
  activeEditorChangeListeners.length = 0;
  refreshedDocuments.length = 0;
  subscriptionHarness.updateContextCount = 0;
  subscriptionHarness.updateContextsResult = false;

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
  await activeEditorChangeListeners[0]?.({ document });

  expect(subscriptionHarness.updateContextCount).toBe(1);
  expect(refreshedDocuments).toEqual([]);
});

it("supported workspace active editor changes update contexts without refreshing diagnostics", async (): Promise<void> => {
  const { registerExtensionSubscriptions } = await import(
    "../../../lifecycle/subscriptions.ts"
  );
  const context = { subscriptions: [] };
  const document = {
    uri: {
      scheme: "file",
      toString: (): string => "file:///workspace/package.json",
    },
  };
  activeEditorChangeListeners.length = 0;
  refreshedDocuments.length = 0;
  subscriptionHarness.updateContextCount = 0;
  subscriptionHarness.updateContextsResult = true;

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
  await activeEditorChangeListeners[0]?.({ document });

  expect(subscriptionHarness.updateContextCount).toBe(1);
  expect(refreshedDocuments).toEqual([]);
});
