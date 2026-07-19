import { readFileSync } from "node:fs";
import process from "node:process";

interface TextDocumentStub {
  getText: () => string;
  languageId: string;
  uri: { toString: () => string };
}

interface AnalyzeOutput {
  canSortDependencies: boolean;
  codeLenses: never[];
  dependencies: never[];
  dependencySignature: string;
  diagnostics: never[];
  installTaskConfigKey: undefined;
  isSupportedManifest: boolean;
  status: {
    dependencyCount: number;
    errorCount: number;
    noMatchCount: number;
    text: string;
    tooltip: string;
    updateCount: number;
    visible: boolean;
    vulnerabilityCount: number;
  };
}

interface ResolveOutput {
  authorizationRequiredCount: number;
  authorizationRequiredRequests: Array<{
    authUrl: string;
    requestUrl: string;
  }>;
  edits: never[];
  suggestions: never[];
  vulnerableUpdateCount: number;
}

interface SessionStub {
  analyzeDocument?: (input: { uri: string }) => AnalyzeOutput;
  applyCommand?: () => undefined;
  clearCache?: () => undefined;
  disposeSession?: () => undefined;
  resolveDocument?: (input?: unknown) => ResolveOutput | Promise<unknown>;
}

interface ExtensionState {
  context: unknown;
  flags: {
    codeLensReplace: boolean;
    providerBusy: number;
    providerError: boolean;
    showOutdated: boolean;
    showPrereleases: boolean;
    showSuggestionStats: boolean;
    showVersionLenses: boolean;
  };
  lifecycle: {
    externalPackageFileWatchers: Map<unknown, unknown>;
    packageFileWatchers: never[];
    sessionGenerations: Map<unknown, unknown>;
  };
  sessions: Map<string, { resource: undefined; session: SessionStub }>;
  snapshots: {
    editedDependencies: Map<string, string>;
    savedDependencies: Map<string, string>;
  };
  ui: {
    diagnostics: { set: (uri: unknown, diagnostics: unknown[]) => void };
    outputChannel: { appendLine: () => undefined };
  };
}

interface AuthenticationContext {
  extensionPath: string;
  secrets: {
    get: (key: string) => string | undefined;
    store: (key: string, value: string) => void;
  };
  storageUri: { path: string };
  workspaceState: {
    get: (key: string, fallback: unknown) => unknown;
    update: (key: string, value: unknown) => void;
  };
}

const registryUrl = "https://registry.example.test";
const authorizationSecret = `/workspace/.vscode__${registryUrl}`;

const diagnosticState = {
  diagnosticSession: {
    activeTextEditor: undefined as { document: TextDocumentStub } | undefined,
    createdSessionConfigs: [] as unknown[],
    diagnosticsSets: [] as { diagnostics: unknown[]; uri: unknown }[],
    reloadedResolveCount: 0,
  },
  userInteraction: {
    inputPrompts: [] as unknown[],
    inputValues: [] as (string | undefined)[],
    quickPickValues: [] as unknown[],
    warningChoice: undefined as string | undefined,
    warningMessages: [] as unknown[],
  },
  configurationAuth: {
    secretValues: {} as Record<string, string | undefined>,
    storedSecrets: [] as { key: string; value: string }[],
    updatedSettings: [] as { key: string; target: boolean; value: unknown }[],
    workspaceConfig: {} as Record<string, unknown>,
    workspaceValues: {} as Record<string, unknown>,
  },
};

function outputFor(uri: string): AnalyzeOutput {
  let statusText = "background status";
  let statusTooltip = "background tooltip";
  if (uri === "file:///workspace/package.json") {
    statusText = "active status";
    statusTooltip = "active tooltip";
  }
  return {
    canSortDependencies: false,
    codeLenses: [],
    dependencies: [],
    dependencySignature: uri,
    diagnostics: [],
    installTaskConfigKey: undefined,
    isSupportedManifest: true,
    status: {
      dependencyCount: 1,
      errorCount: 0,
      noMatchCount: 0,
      text: statusText,
      tooltip: statusTooltip,
      updateCount: 1,
      visible: true,
      vulnerabilityCount: 0,
    },
  };
}

function defaultSession(): Required<SessionStub> {
  return {
    analyzeDocument: (input: { uri: string }): AnalyzeOutput =>
      outputFor(input.uri),
    applyCommand: (): undefined => undefined,
    clearCache: (): undefined => undefined,
    disposeSession: (): undefined => undefined,
    resolveDocument: (): ResolveOutput => ({
      authorizationRequiredCount: 0,
      authorizationRequiredRequests: [],
      edits: [],
      suggestions: [],
      vulnerableUpdateCount: 0,
    }),
  };
}

function createExtensionState(
  extra: Record<string, unknown> = {},
): ExtensionState {
  const { session = defaultSession(), ...remaining } = extra;
  return {
    context: undefined as unknown,
    flags: {
      codeLensReplace: true,
      providerBusy: 0,
      providerError: false,
      showOutdated: false,
      showPrereleases: false,
      showSuggestionStats: false,
      showVersionLenses: true,
    },
    lifecycle: {
      externalPackageFileWatchers: new Map(),
      packageFileWatchers: [] as never[],
      sessionGenerations: new Map(),
    },
    sessions: new Map([
      ["global", { resource: undefined, session: session as SessionStub }],
    ]),
    snapshots: {
      editedDependencies: new Map<string, string>(),
      savedDependencies: new Map<string, string>(),
    },
    ui: {
      diagnostics: {
        set(uri: unknown, diagnostics: unknown[]): void {
          diagnosticState.diagnosticSession.diagnosticsSets.push({
            diagnostics,
            uri,
          });
        },
      },
      outputChannel: { appendLine: (): undefined => undefined },
    },
    ...remaining,
  };
}

function authContext(): AuthenticationContext {
  return {
    extensionPath: "/test/extension",
    secrets: {
      get: (key: string): string | undefined =>
        diagnosticState.configurationAuth.secretValues[key],
      store(key: string, value: string): void {
        diagnosticState.configurationAuth.secretValues[key] = value;
        diagnosticState.configurationAuth.storedSecrets.push({ key, value });
      },
    },
    storageUri: { path: "/workspace/.vscode" },
    workspaceState: {
      get: (key: string, fallback: unknown): unknown =>
        diagnosticState.configurationAuth.workspaceValues[key] ?? fallback,
      update: (key: string, value: unknown): void => {
        diagnosticState.configurationAuth.workspaceValues[key] = value;
        diagnosticState.configurationAuth.updatedSettings.push({
          key,
          target: false,
          value,
        });
      },
    },
  };
}

function documentStub(uri: string): TextDocumentStub {
  return {
    getText: (): string => packageFileFixture("empty.json"),
    languageId: "json",
    uri: { toString: (): string => uri },
  };
}

function reset(): void {
  diagnosticState.diagnosticSession.activeTextEditor = undefined;
  diagnosticState.diagnosticSession.reloadedResolveCount = 0;
  diagnosticState.userInteraction.warningChoice = undefined;
  for (const values of [
    diagnosticState.diagnosticSession.createdSessionConfigs,
    diagnosticState.diagnosticSession.diagnosticsSets,
    diagnosticState.userInteraction.inputValues,
    diagnosticState.userInteraction.inputPrompts,
    diagnosticState.userInteraction.quickPickValues,
    diagnosticState.configurationAuth.storedSecrets,
    diagnosticState.configurationAuth.updatedSettings,
    diagnosticState.userInteraction.warningMessages,
  ]) {
    values.length = 0;
  }
  for (const record of [
    diagnosticState.configurationAuth.workspaceConfig,
    diagnosticState.configurationAuth.workspaceValues,
    diagnosticState.configurationAuth.secretValues,
  ]) {
    for (const key of Object.keys(record)) {
      delete record[key];
    }
  }
}

function packageFileFixture(name: string): string {
  return readFileSync(
    `${process.cwd()}/tests/fixtures/vscode-extension/${name}`,
    "utf8",
  );
}

export type { AnalyzeOutput, ResolveOutput, SessionStub, TextDocumentStub };
export {
  authContext,
  authorizationSecret,
  createExtensionState,
  diagnosticState,
  documentStub,
  outputFor,
  registryUrl,
  reset,
};
