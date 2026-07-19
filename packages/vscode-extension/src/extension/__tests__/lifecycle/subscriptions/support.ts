import { mock, mockVscodeHost } from "../../runtime.ts";

type MockModule = Record<string, unknown>;

interface EditorStub {
  document: unknown;
}

interface TextChangeEvent {
  contentChanges?: unknown[];
  document: unknown;
  reason?: number;
}

const subscriptionHarness: {
  activeTextEditor: EditorStub | undefined;
  analyzeDocumentResult: { isSupportedManifest: boolean } | undefined;
  updateContextCount: number;
  updateContextsResult: boolean;
} = {
  activeTextEditor: undefined,
  analyzeDocumentResult: undefined,
  updateContextCount: 0,
  updateContextsResult: false,
};
const refreshedDocuments: unknown[] = [];
const textDocumentChangeListeners: Array<
  (event: TextChangeEvent) => Promise<void> | void
> = [];
const textDocumentCloseListeners: Array<(document: { uri: unknown }) => void> =
  [];
const activeEditorChangeListeners: Array<
  (editor: EditorStub | undefined) => Promise<void> | void
> = [];
const createdWatcherPatterns: unknown[] = [];

function relativePattern(
  this: { base: unknown; pattern: string },
  base: unknown,
  pattern: string,
): void {
  this.base = base;
  this.pattern = pattern;
}

function windowMock(): MockModule {
  return {
    get activeTextEditor(): EditorStub | undefined {
      return subscriptionHarness.activeTextEditor;
    },
    onDidChangeActiveTextEditor(
      listener: (editor: EditorStub | undefined) => Promise<void> | void,
    ): { dispose: () => undefined } {
      activeEditorChangeListeners.push(listener);
      return { dispose: (): undefined => undefined };
    },
  };
}

function fileSystemWatcher(pattern: unknown): MockModule {
  createdWatcherPatterns.push(pattern);
  const registration = (): { dispose: () => undefined } => ({
    dispose: (): undefined => undefined,
  });
  return {
    dispose: (): undefined => undefined,
    onDidChange: registration,
    onDidCreate: registration,
    onDidDelete: registration,
  };
}

function workspaceMock(): MockModule {
  return {
    workspaceFolders: [{ uri: { fsPath: "/workspace" } }],
    createFileSystemWatcher: fileSystemWatcher,
    getWorkspaceFolder: (): { uri: { fsPath: string } } => ({
      uri: { fsPath: "/workspace" },
    }),
    getConfiguration: (): MockModule => ({
      get: (key: string, fallback?: unknown): unknown => {
        if (key === "npm.files") {
          return "**/package.json";
        }
        return fallback;
      },
    }),
    onDidChangeTextDocument(
      listener: (event: TextChangeEvent) => Promise<void> | void,
    ): { dispose: () => undefined } {
      textDocumentChangeListeners.push(listener);
      return { dispose: (): undefined => undefined };
    },
    onDidSaveTextDocument: (): { dispose: () => undefined } => ({
      dispose: (): undefined => undefined,
    }),
    onDidCloseTextDocument(listener: (document: { uri: unknown }) => void): {
      dispose: () => undefined;
    } {
      textDocumentCloseListeners.push(listener);
      return { dispose: (): undefined => undefined };
    },
  };
}

function vscodeMock(): MockModule {
  return Object.fromEntries([
    ["RelativePattern", relativePattern],
    [
      "TextDocumentChangeReason",
      Object.fromEntries([
        ["Redo", 2],
        ["Undo", 1],
      ]),
    ],
    ["window", windowMock()],
    ["workspace", workspaceMock()],
  ]);
}

function updateContext(): boolean {
  subscriptionHarness.updateContextCount += 1;
  return subscriptionHarness.updateContextsResult;
}

mockVscodeHost(vscodeMock);
mock.module(
  "../../../commands/register.ts",
  (): MockModule => ({
    registerCommands: (): never[] => [],
  }),
);
mock.module(
  "../../../commands/contexts.ts",
  (): MockModule => ({
    updateContexts: updateContext,
  }),
);
mock.module(
  "../../../diagnostics/refresh.ts",
  (): MockModule => ({
    refreshDiagnostics: (_state: unknown, document: unknown): void => {
      refreshedDocuments.push(document);
    },
  }),
);
mock.module(
  "../../../diagnostics/analyze.ts",
  (): MockModule => ({
    analyzeDocument: (): { isSupportedManifest: boolean } | undefined =>
      subscriptionHarness.analyzeDocumentResult,
  }),
);
mock.module(
  "../../../tasks.ts",
  (): MockModule => ({
    handleDidSaveTextDocument: (): undefined => undefined,
  }),
);
mock.module(
  "../../../tasks/custom-install.ts",
  (): MockModule => ({
    customInstallTaskLabel: (): undefined => undefined,
  }),
);
mock.module(
  "../../../lifecycle/refresh-timer.ts",
  (): MockModule => ({
    registerRefreshTimer: (): { dispose: () => undefined } => ({
      dispose: (): undefined => undefined,
    }),
  }),
);

export {
  activeEditorChangeListeners,
  createdWatcherPatterns,
  refreshedDocuments,
  subscriptionHarness,
  textDocumentChangeListeners,
  textDocumentCloseListeners,
};
