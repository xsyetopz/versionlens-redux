import { mock, mockVscodeHost } from "../../runtime.ts";
import type {
  Disposable,
  DocumentUri,
  TestConfiguration,
  TestDocument,
  TestUri,
  TestWorkspaceFolder,
} from "./contracts.ts";
import { packageFileFixture } from "./fixture.ts";

interface Watcher {
  changed: ((uri: unknown) => Promise<void> | void)[];
  created: ((uri: unknown) => Promise<void> | void)[];
  deleted: ((uri: unknown) => Promise<void> | void)[];
  dispose: () => void;
  disposeCount: number;
  onDidChange: (listener: (uri: unknown) => Promise<void> | void) => Disposable;
  onDidCreate: (listener: (uri: unknown) => Promise<void> | void) => Disposable;
  onDidDelete: (listener: (uri: unknown) => Promise<void> | void) => Disposable;
  pattern: unknown;
}

const createdWatchers: Watcher[] = [];
const findFilesCalls: { exclude?: unknown; include: unknown }[] = [];
const openedDocuments: unknown[] = [];
const analyzedInputs: unknown[] = [];
const refreshedDocuments: unknown[] = [];
let codeLensRefreshCount = 0;
let activeTextEditor: { document: unknown } | undefined;
let workspaceFolders: { uri: { fsPath: string } }[] | undefined = [
  { uri: { fsPath: "/workspace" } },
];
const filesConfig: Record<string, unknown> = {
  exclude: { "**/dist/**": true, "**/tmp/**": false },
};
const versionlensConfig: Record<string, unknown> & {
  enabledProviders: unknown | undefined;
} = {
  enabledProviders: undefined,
  "hex.files": "**/{mix.exs,rebar.config,gleam.toml}",
  "npm.files": "**/package.json",
};

function uri(value: string): TestUri {
  const uriValue = value;
  return {
    fsPath: uriValue.replace("file://", ""),
    scheme: uriValue.split(":")[0],
    toString: (): string => uriValue,
  };
}

function document(
  value: string,
  text = packageFileFixture("package-left-pad.json"),
): TestDocument {
  const documentValue = value;
  const documentText = text;
  return {
    getText: (): string => documentText,
    isDirty: false,
    languageId: "json",
    uri: uri(documentValue),
  };
}

function relativePattern(
  this: { base: unknown; pattern: string },
  base: unknown,
  pattern: string,
): void {
  this.base = base;
  this.pattern = pattern;
}

function fileWatcher(pattern: unknown): Watcher {
  const watcher: Watcher = {
    changed: [],
    created: [],
    deleted: [],
    dispose: (): void => {
      watcher.disposeCount += 1;
    },
    disposeCount: 0,
    onDidChange: (
      listener: (uri: unknown) => Promise<void> | void,
    ): Disposable => listenerRegistration(watcher.changed, listener),
    onDidCreate: (
      listener: (uri: unknown) => Promise<void> | void,
    ): Disposable => listenerRegistration(watcher.created, listener),
    onDidDelete: (
      listener: (uri: unknown) => Promise<void> | void,
    ): Disposable => listenerRegistration(watcher.deleted, listener),
    pattern,
  };
  createdWatchers.push(watcher);
  return watcher;
}

function listenerRegistration(
  listeners: ((uri: unknown) => Promise<void> | void)[],
  listener: (uri: unknown) => Promise<void> | void,
): Disposable {
  listeners.push(listener);
  return { dispose: (): undefined => undefined };
}

function configuration(section?: string): TestConfiguration {
  const configurationSection = section;
  let values: Record<string, unknown> = versionlensConfig;
  if (configurationSection === "files") {
    values = filesConfig;
  }
  return {
    get(key: string, fallback?: unknown): unknown {
      if (Object.hasOwn(values, key)) {
        return values[key];
      }
      return fallback;
    },
  };
}

function workspaceFolder(
  documentUri: DocumentUri,
): TestWorkspaceFolder | undefined {
  const resourceUri = documentUri;
  if (!resourceUri.fsPath?.startsWith("/workspace/")) {
    return;
  }
  return workspaceFolders?.[0];
}

function vscodeMock(): Record<string, unknown> {
  return Object.fromEntries([
    ["RelativePattern", relativePattern],
    ["commands", { executeCommand: (): undefined => undefined }],
    [
      "window",
      {
        get activeTextEditor(): { document: unknown } | undefined {
          return activeTextEditor;
        },
      },
    ],
    [
      "workspace",
      {
        createFileSystemWatcher: fileWatcher,
        findFiles(
          include: unknown,
          exclude?: unknown,
        ): ReturnType<typeof uri>[] {
          findFilesCalls.push({ exclude, include });
          return [uri("file:///workspace/package.json")];
        },
        getConfiguration: configuration,
        get workspaceFolders(): typeof workspaceFolders {
          return workspaceFolders;
        },
        getWorkspaceFolder: workspaceFolder,
        openTextDocument(openedUri: unknown): ReturnType<typeof document> {
          openedDocuments.push(openedUri);
          return document((openedUri as { toString: () => string }).toString());
        },
      },
    ],
  ]);
}

mockVscodeHost(vscodeMock);
mock.module(
  "../../../commands/codelens.ts",
  (): Record<string, unknown> => ({
    refreshCodeLenses: (): void => {
      codeLensRefreshCount += 1;
    },
  }),
);
mock.module(
  "../../../commands/contexts.ts",
  (): Record<string, unknown> => ({
    updateContexts: (): undefined => undefined,
  }),
);
mock.module(
  "../../../diagnostics/analyze.ts",
  (): Record<string, unknown> => ({
    analyzeDocument: (
      _state: unknown,
      documentInput: ReturnType<typeof document>,
    ) => {
      const folder = workspaceFolder(documentInput.uri);
      const input = {
        languageId: documentInput.languageId,
        text: documentInput.getText(),
        uri: documentInput.uri.toString(),
      };
      if (folder) {
        Object.assign(input, { workspaceRoot: folder.uri.fsPath });
      }
      analyzedInputs.push(input);
      return {
        codeLenses: [],
        dependencySignature: `signature-${analyzedInputs.length}`,
        diagnostics: [],
        isSupportedManifest: true,
      };
    },
  }),
);
mock.module(
  "../../../diagnostics/refresh.ts",
  (): Record<string, unknown> => ({
    refreshDiagnostics: (_state: unknown, currentDocument: unknown): void => {
      refreshedDocuments.push(currentDocument);
    },
  }),
);

function state(): {
  flags: {
    providerBusy: number;
    providerError: boolean;
    showVersionLenses: boolean;
  };
  lifecycle: {
    externalPackageFileWatchers: Map<string, Disposable[]>;
    packageFileWatchers: Disposable[];
  };
  snapshots: {
    editedDependencies: Map<string, string>;
    savedDependencies: Map<string, string>;
  };
  ui: object;
} {
  return {
    flags: { providerBusy: 0, providerError: false, showVersionLenses: true },
    lifecycle: {
      externalPackageFileWatchers: new Map(),
      packageFileWatchers: [],
    },
    snapshots: { editedDependencies: new Map(), savedDependencies: new Map() },
    ui: {
      codeLensRefresh: {
        fire: (): void => {
          codeLensRefreshCount += 1;
        },
      },
      diagnostics: { set: (): undefined => undefined },
      outputChannel: {},
    },
  };
}

function reset(): void {
  createdWatchers.length = 0;
  findFilesCalls.length = 0;
  openedDocuments.length = 0;
  analyzedInputs.length = 0;
  refreshedDocuments.length = 0;
  codeLensRefreshCount = 0;
  activeTextEditor = undefined;
  workspaceFolders = [{ uri: { fsPath: "/workspace" } }];
  versionlensConfig.enabledProviders = undefined;
}

function setActiveTextEditor(value: { document: unknown } | undefined): void {
  activeTextEditor = value;
}

function setWorkspaceFolders(value: typeof workspaceFolders): void {
  workspaceFolders = value;
}

function globPattern(value: unknown): string {
  if (typeof value === "string") {
    return value;
  }
  return (value as { pattern: string }).pattern;
}

export {
  analyzedInputs,
  codeLensRefreshCount,
  createdWatchers,
  document,
  findFilesCalls,
  globPattern,
  openedDocuments,
  refreshedDocuments,
  reset,
  setActiveTextEditor,
  setWorkspaceFolders,
  state,
  uri,
  versionlensConfig,
};
