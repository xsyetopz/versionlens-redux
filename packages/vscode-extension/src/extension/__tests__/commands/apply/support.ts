import { mock, mockVscodeHost } from "../../runtime.ts";
import {
  codeLens,
  range,
  relativePattern,
  workspaceEdit,
} from "./constructors.ts";
import { clearRecord } from "./records.ts";
import { createWorkspaceMock } from "./workspace-mock.ts";

type MockModule = Record<string, unknown>;
interface Disposable {
  dispose: () => undefined;
}
interface EventEmitterMock {
  dispose: () => void;
  event: () => Disposable;
  fire: () => void;
}
interface NativeSessionStub {
  analyzeDocument: () => undefined;
  applyCommand: () => undefined;
  clearCache: () => undefined;
  disposeSession: () => undefined;
  resolveDocument: () => undefined;
}
interface CommandsMock {
  executeCommand: () => undefined;
  registerCommand: (
    command: string,
    callback: (...args: unknown[]) => unknown,
  ) => Disposable;
}
interface WindowMock {
  readonly activeTextEditor: { document: unknown } | undefined;
  onDidChangeActiveTextEditor: () => Disposable;
  showInputBox: (options: unknown) => string | undefined;
  showQuickPick: () => unknown;
  showWarningMessage: (
    ...args: unknown[]
  ) => string | Promise<string | undefined> | undefined;
}

const applyTestState: {
  activeTextEditor: { document: unknown } | undefined;
  applyEditBlocker: Promise<boolean> | undefined;
  codeLensRefreshCount: number;
  warningChoice: Promise<string | undefined> | string | undefined;
} = {
  activeTextEditor: undefined,
  applyEditBlocker: undefined,
  codeLensRefreshCount: 0,
  warningChoice: undefined,
};
const appliedEdits: unknown[] = [];
const createdSessionConfigs: unknown[] = [];
const createdNativeSessions: unknown[] = [];
const inputValues: string[] = [];
const inputPrompts: unknown[] = [];
const outputLines: string[] = [];
const quickPickValues: unknown[] = [];
const registeredCommands: Record<
  string,
  (...args: unknown[]) => Promise<unknown>
> = {};
const storedSecrets: { key: string; value: string }[] = [];
const updatedConfig: { key: string; target: boolean; value: unknown }[] = [];
const warningMessages: unknown[] = [];
const workspaceConfig: Record<string, unknown> = {};
const workspaceValues: Record<string, unknown> = {};
const secretValues: Record<string, string | undefined> = {};
const registryUrl = "https://registry.example.test";
const authorizationSecret = `/workspace/.vscode__${registryUrl}`;

function eventEmitter(this: EventEmitterMock): void {
  this.event = (): Disposable => ({ dispose: (): undefined => undefined });
  this.dispose = (): undefined => undefined;
  this.fire = (): void => {
    applyTestState.codeLensRefreshCount += 1;
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

function windowMock(): WindowMock {
  return {
    get activeTextEditor(): { document: unknown } | undefined {
      return applyTestState.activeTextEditor;
    },
    onDidChangeActiveTextEditor: (): Disposable => ({
      dispose: (): undefined => undefined,
    }),
    showInputBox: (options: unknown): string | undefined => {
      inputPrompts.push(options);
      return inputValues.shift();
    },
    showQuickPick: (): unknown => quickPickValues.shift(),
    showWarningMessage: (
      ...args: unknown[]
    ): string | Promise<string | undefined> | undefined => {
      warningMessages.push(args);
      return applyTestState.warningChoice;
    },
  };
}

function vscodeMock(): MockModule {
  const workspace = createWorkspaceMock({
    appliedEdits,
    state: applyTestState,
    updatedConfig,
    workspaceConfig,
  });
  return Object.fromEntries([
    ["CodeLens", codeLens],
    ["Diagnostic", codeLens],
    ["EventEmitter", eventEmitter],
    ["FileType", Object.fromEntries([["Directory", 2]])],
    ["Range", range],
    ["RelativePattern", relativePattern],
    [
      "TextDocumentChangeReason",
      Object.fromEntries([
        ["Redo", 2],
        ["Undo", 1],
      ]),
    ],
    [
      "Uri",
      {
        file: (path: string): { path: string; scheme: string } => ({
          path,
          scheme: "file",
        }),
      },
    ],
    ["WorkspaceEdit", workspaceEdit],
    ["commands", commandsMock()],
    ["env", { openExternal: (): undefined => undefined }],
    ["extensions", { getExtension: (): undefined => undefined }],
    [
      "languages",
      {
        registerCodeLensProvider: (): Disposable => ({
          dispose: (): undefined => undefined,
        }),
      },
    ],
    [
      "tasks",
      {
        executeTask: (): undefined => undefined,
        fetchTasks: (): never[] => [],
      },
    ],
    ["window", windowMock()],
    ["workspace", { ...workspace.events, ...workspace.operations }],
  ]);
}

mockVscodeHost(vscodeMock);
mock.module(
  "../../../diagnostics/refresh.ts",
  (): MockModule => ({
    analyzeDocument: (): undefined => undefined,
    dependencySnapshot: (): string => "",
    refreshActiveDiagnostics: (): undefined => undefined,
    refreshDiagnostics: (): undefined => undefined,
    setProviderState: (): undefined => undefined,
  }),
);
mock.module(
  "../../../native/module.ts",
  (): MockModule => ({
    loadNative: (): { createSession: (config: unknown) => object } => ({
      createSession(config: unknown): object {
        createdSessionConfigs.push(config);
        return createdNativeSessions.shift() ?? defaultNativeSession();
      },
    }),
  }),
);

function defaultNativeSession(): NativeSessionStub {
  return {
    analyzeDocument: (): undefined => undefined,
    applyCommand: (): undefined => undefined,
    clearCache: (): undefined => undefined,
    disposeSession: (): undefined => undefined,
    resolveDocument: (): undefined => undefined,
  };
}

function registeredCommand(
  command: string,
): (...args: unknown[]) => Promise<unknown> {
  const callback = registeredCommands[command];
  if (!callback) {
    throw new Error(`command not registered: ${command}`);
  }
  return callback;
}

function reset(): void {
  applyTestState.activeTextEditor = undefined;
  applyTestState.applyEditBlocker = undefined;
  applyTestState.codeLensRefreshCount = 0;
  applyTestState.warningChoice = undefined;
  for (const values of [
    appliedEdits,
    createdSessionConfigs,
    createdNativeSessions,
    inputValues,
    inputPrompts,
    outputLines,
    quickPickValues,
    storedSecrets,
    updatedConfig,
    warningMessages,
  ]) {
    values.length = 0;
  }
  clearRecord(workspaceConfig);
  clearRecord(workspaceValues);
  clearRecord(secretValues);
  clearRecord(registeredCommands);
}

export {
  appliedEdits,
  applyTestState,
  authorizationSecret,
  createdNativeSessions,
  createdSessionConfigs,
  inputPrompts,
  inputValues,
  outputLines,
  quickPickValues,
  registeredCommand,
  registeredCommands,
  registryUrl,
  reset,
  secretValues,
  storedSecrets,
  updatedConfig,
  warningMessages,
  workspaceConfig,
  workspaceValues,
};
