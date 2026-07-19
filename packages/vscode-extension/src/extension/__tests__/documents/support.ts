import { mockVscodeHost } from "../runtime.ts";
import { packageFileFixture } from "./fixture.ts";
import { defaultFilePatternByKey } from "./patterns.ts";

type MockModule = Record<string, unknown>;
interface Disposable {
  dispose: () => void;
}
interface CodeLensProvider {
  onDidChangeCodeLenses?: (listener: () => void) => Disposable;
  provideCodeLenses: (document: unknown) => unknown[];
}
interface CommandsMock {
  executeCommand: () => undefined;
  registerCommand: (
    command: string,
    callback: (...args: unknown[]) => unknown,
  ) => Disposable;
}
interface LanguagesMock {
  registerCodeLensProvider: (
    selector: unknown,
    provider: CodeLensProvider,
  ) => Disposable;
}
interface WorkspaceMock {
  applyEdit: (edit: { edits: unknown[] }) => boolean;
  getConfiguration: () => {
    get: (key: string, fallback?: unknown) => unknown;
  };
  getWorkspaceFolder: () => undefined;
  onDidChangeConfiguration: () => Disposable;
  onDidChangeWorkspaceFolders: () => Disposable;
}
interface AnalysisOutput {
  canSortDependencies: boolean;
  codeLenses: unknown[];
  dependencies: unknown[];
  dependencySignature: string;
  diagnostics: unknown[];
  isSupportedManifest: boolean;
  status: { text: string; tooltip: string; visible: boolean };
}
interface TestDocument {
  getText: () => string;
  languageId: string;
  uri: { toString: () => string };
  version?: number;
}
interface CodeLensState {
  flags: {
    providerBusy: number;
    providerError: boolean;
    showVersionLenses: boolean;
  };
  sessions: Map<string, { resource: undefined; session: object }>;
  ui: { codeLensRefresh: undefined };
}
interface DocumentConfiguration
  extends Record<string, string | string[] | undefined> {
  enabledProviders: string[] | undefined;
}

interface MockEventEmitter {
  dispose: () => void;
  event: (listener: () => void) => Disposable;
  fire: () => void;
  listeners: (() => void)[];
}

interface MockWorkspaceEdit {
  edits: unknown[];
  replace: (uri: unknown, range: unknown, newText: string) => void;
}

const configured: DocumentConfiguration = { enabledProviders: undefined };
const codeLensProviders: CodeLensProvider[] = [];
const diagnosticsSets: { diagnostics: unknown[]; uri: unknown }[] = [];
let activeTextEditor: { document: unknown } | undefined;
const testGlobals = globalThis as typeof globalThis & {
  __versionLensAppliedEdits?: unknown[];
  __versionLensRegisteredCommands?: Record<
    string,
    (...args: unknown[]) => unknown
  >;
};
testGlobals.__versionLensRegisteredCommands ??= {};
testGlobals.__versionLensAppliedEdits ??= [];
const registeredCommands: Record<string, (...args: unknown[]) => unknown> =
  testGlobals.__versionLensRegisteredCommands;
const appliedEdits: unknown[] = testGlobals.__versionLensAppliedEdits;

function mockCodeLens(
  this: { command: unknown; range: unknown },
  range: unknown,
  command: unknown,
): void {
  this.range = range;
  this.command = command;
}

function mockDiagnostic(
  this: { message: string; range: unknown; severity: number },
  range: unknown,
  message: string,
  severity: number,
): void {
  this.range = range;
  this.message = message;
  this.severity = severity;
}

function mockEventEmitter(this: MockEventEmitter): void {
  this.listeners = [];
  this.event = (listener: () => void): Disposable => {
    this.listeners.push(listener);
    return { dispose: (): void => removeListener(this.listeners, listener) };
  };
  this.fire = (): void => {
    for (const listener of [...this.listeners]) {
      listener();
    }
  };
  this.dispose = (): void => {
    this.listeners = [];
  };
}

function removeListener(listeners: (() => void)[], listener: () => void): void {
  const index = listeners.indexOf(listener);
  if (index >= 0) {
    listeners.splice(index, 1);
  }
}

function mockRange(this: { values: number[] }, ...values: number[]): void {
  this.values = values;
}

function mockRelativePattern(
  this: { base: unknown; pattern: string },
  base: unknown,
  pattern: string,
): void {
  this.base = base;
  this.pattern = pattern;
}

function mockWorkspaceEdit(this: MockWorkspaceEdit): void {
  this.edits = [];
  this.replace = (uri: unknown, range: unknown, newText: string): void => {
    this.edits.push({ newText, range, uri });
  };
}

function commandsMock(): CommandsMock {
  return {
    executeCommand: (): undefined => undefined,
    registerCommand(
      command: string,
      callback: (...args: unknown[]) => unknown,
    ): Disposable {
      registeredCommands[command] = async (
        ...args: unknown[]
      ): Promise<unknown> => callback(...args);
      return { dispose: (): undefined => undefined };
    },
  };
}

function languagesMock(): LanguagesMock {
  return {
    registerCodeLensProvider(
      _: unknown,
      provider: CodeLensProvider,
    ): Disposable {
      codeLensProviders.push(provider);
      return { dispose: (): undefined => undefined };
    },
  };
}

function workspaceMock(): WorkspaceMock {
  return {
    applyEdit(edit: { edits: unknown[] }): boolean {
      appliedEdits.push(...edit.edits);
      return true;
    },
    getConfiguration(): { get: (key: string, fallback?: unknown) => unknown } {
      return {
        get: (key: string, fallback?: unknown): unknown =>
          configured[key] ?? defaultFilePatternByKey.get(key) ?? fallback,
      };
    },
    getWorkspaceFolder: (): undefined => undefined,
    onDidChangeConfiguration: (): Disposable => ({
      dispose: (): undefined => undefined,
    }),
    onDidChangeWorkspaceFolders: (): Disposable => ({
      dispose: (): undefined => undefined,
    }),
  };
}

function vscodeMock(): MockModule {
  return Object.fromEntries([
    ["CodeLens", mockCodeLens],
    ["Diagnostic", mockDiagnostic],
    ["EventEmitter", mockEventEmitter],
    ["Range", mockRange],
    ["RelativePattern", mockRelativePattern],
    [
      "Uri",
      {
        file: (path: string): { path: string; scheme: string } => ({
          path,
          scheme: "file",
        }),
        parse: (
          value: string,
        ): { scheme: string | undefined; value: string } => ({
          scheme: value.split(":")[0],
          value,
        }),
      },
    ],
    ["WorkspaceEdit", mockWorkspaceEdit],
    ["commands", commandsMock()],
    ["env", { openExternal: (): undefined => undefined }],
    ["languages", languagesMock()],
    [
      "tasks",
      {
        executeTask: (): undefined => undefined,
        fetchTasks: (): never[] => [],
      },
    ],
    [
      "window",
      {
        get activeTextEditor(): { document: unknown } | undefined {
          return activeTextEditor;
        },
        showWarningMessage: (): undefined => undefined,
      },
    ],
    ["workspace", workspaceMock()],
  ]);
}

function setActiveTextEditor(value: { document: unknown } | undefined): void {
  activeTextEditor = value;
}

function analysisOutput(): AnalysisOutput {
  return {
    canSortDependencies: true,
    codeLenses: [],
    dependencies: [],
    dependencySignature: "left-pad@1.0.0",
    diagnostics: [],
    isSupportedManifest: true,
    status: { text: "Version Lens", tooltip: "", visible: true },
  };
}

function testDocument(uri = "file:///package.json"): TestDocument {
  return {
    getText: (): string => packageFileFixture("package-left-pad.json"),
    languageId: "json",
    uri: { toString: (): string => uri },
  };
}

function codeLensState(session: object): CodeLensState {
  return {
    flags: {
      providerBusy: 0,
      providerError: false,
      showVersionLenses: true,
    },
    sessions: new Map([["global", { resource: undefined, session }]]),
    ui: { codeLensRefresh: undefined },
  };
}

mockVscodeHost(vscodeMock);

export {
  analysisOutput,
  appliedEdits,
  codeLensProviders,
  codeLensState,
  configured,
  diagnosticsSets,
  registeredCommands,
  setActiveTextEditor,
  testDocument,
};
