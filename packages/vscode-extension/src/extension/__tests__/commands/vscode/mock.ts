import { packageFileFixture } from "../fixture.ts";
import {
  createCodeLensConstructor,
  createEventEmitterConstructor,
  createRelativePatternConstructor,
  createWorkspaceEditConstructor,
} from "./constructors.ts";
import type {
  CommandsMockContext,
  LanguagesMockContext,
  TaskExecution,
  TasksMockContext,
  VscodeMockContext,
  VscodeMockState,
  WindowMockContext,
  WorkspaceMockContext,
} from "./context.ts";

type MockModule = Record<string, unknown>;

interface FileSystemWatcherMock {
  dispose: () => void;
  onDidChange: () => { dispose: () => undefined };
  onDidCreate: () => { dispose: () => undefined };
  onDidDelete: () => { dispose: () => undefined };
}

interface WorkspaceConfigurationMock {
  get: (key: string, fallback?: unknown) => unknown;
  inspect: (key: string) => { workspaceValue: unknown | null } | undefined;
  update: (key: string, value: unknown) => void;
}

function createCommandsMock(context: CommandsMockContext): MockModule {
  return {
    executeCommand(command: string, key: string, value: unknown): void {
      if (command === "setContext") {
        context.contexts[key] = value;
      }
    },
    registerCommand(
      command: string,
      callback: (...args: unknown[]) => unknown,
    ): { dispose: () => undefined } {
      context.registeredCommands[command] = callback;
      return { dispose: (): undefined => undefined };
    },
  };
}

function createLanguagesMock(context: LanguagesMockContext): MockModule {
  return {
    registerCodeLensProvider(selector: unknown): { dispose: () => void } {
      context.codeLensSelectors.push(selector);
      let disposed = false;
      return {
        dispose(): void {
          if (!disposed) {
            disposed = true;
            context.codeLensRegistrationDisposals.push(1);
          }
        },
      };
    },
  };
}

function createTasksMock(
  context: TasksMockContext,
  state: VscodeMockState,
): MockModule {
  return {
    executeTask(task: { name: string }): TaskExecution {
      context.executedTasks.push(task.name);
      const execution = { task, terminate: (): undefined => undefined };
      context.taskExecutions.push(execution);
      if (state.taskCompletionMode === "auto") {
        queueMicrotask((): void => {
          context.completeTaskExecution(execution, 0);
        });
      }
      return execution;
    },
    fetchTasks(): Array<{ name: string }> {
      return [{ name: context.smokeTaskLabel }];
    },
    onDidEndTaskProcess(
      listener: (event: {
        execution: { task: { name: string } };
        exitCode: number | undefined;
      }) => void,
    ): { dispose: () => void } {
      context.taskEndListeners.push(listener);
      return {
        dispose(): void {
          const index = context.taskEndListeners.indexOf(listener);
          if (index >= 0) {
            context.taskEndListeners.splice(index, 1);
          }
        },
      };
    },
  };
}

function createWindowMock(
  context: WindowMockContext,
  state: VscodeMockState,
): MockModule {
  return {
    get activeTextEditor(): { document: unknown } | undefined {
      return state.activeTextEditor;
    },
    onDidChangeActiveTextEditor: (): { dispose: () => undefined } => ({
      dispose: (): undefined => undefined,
    }),
    showInformationMessage(...args: unknown[]): void {
      context.warningMessages.push(args);
    },
    showInputBox: (): undefined => undefined,
    showQuickPick(items: unknown[], options?: unknown): unknown {
      context.quickPickItems.push(...items);
      context.quickPickOptions.push(options);
      return items[0];
    },
    showTextDocument(uri: unknown): void {
      context.shownTextDocuments.push(uri);
    },
    showWarningMessage(...args: unknown[]): string | undefined {
      context.warningMessages.push(args);
      return state.warningChoice;
    },
  };
}

function createFileSystemWatcher(
  context: WorkspaceMockContext,
  pattern: unknown,
): FileSystemWatcherMock {
  const { fileSystemWatchers } = context;
  const watcherPattern = pattern;
  const watcher = { disposed: false, pattern: watcherPattern };
  const event = (): { dispose: () => undefined } => ({
    dispose: (): undefined => undefined,
  });
  fileSystemWatchers.push(watcher);
  return {
    dispose(): void {
      watcher.disposed = true;
    },
    onDidChange: event,
    onDidCreate: event,
    onDidDelete: event,
  };
}

function createWorkspaceConfiguration(
  workspaceConfig: Record<string, unknown>,
): WorkspaceConfigurationMock {
  const configurationValues = workspaceConfig;
  return {
    get(key: string, fallback?: unknown): unknown {
      if (Object.hasOwn(configurationValues, key)) {
        return configurationValues[key];
      }
      return fallback;
    },
    inspect(key: string): { workspaceValue: unknown | null } | undefined {
      const value = configurationValues[key];
      if (value === undefined) {
        return;
      }
      return { workspaceValue: value };
    },
    update(key: string, value: unknown): void {
      configurationValues[key] = value;
    },
  };
}

function createWorkspaceMock(
  context: WorkspaceMockContext,
  state: VscodeMockState,
): MockModule {
  const disposableEvent = (): { dispose: () => undefined } => ({
    dispose: (): undefined => undefined,
  });
  const workspaceUri = {
    fsPath: "/workspace",
    toString: (): string => "file:///workspace",
  };
  return {
    applyEdit(edit: { edits: unknown[] }): boolean {
      context.appliedEdits.push(...edit.edits);
      return true;
    },
    asRelativePath(uri: { toString: () => string }): string {
      return uri.toString().replace("file:///workspace/", "");
    },
    createFileSystemWatcher: (pattern: unknown) =>
      createFileSystemWatcher(context, pattern),
    findFiles(
      include: unknown,
      exclude?: unknown,
    ): Array<{ fsPath: string; toString: () => string }> {
      context.findFilesCalls.push({ exclude, include });
      return [
        {
          fsPath: "/workspace/package.json",
          toString: (): string => "file:///workspace/package.json",
        },
      ];
    },
    fs: {
      stat: (): { type: number } => ({
        type: state.dependencyFileType,
      }),
    },
    getConfiguration: () =>
      createWorkspaceConfiguration(context.workspaceConfig),
    getWorkspaceFolder: (): { uri: typeof workspaceUri } => ({
      uri: workspaceUri,
    }),
    onDidChangeConfiguration(
      listener: WorkspaceMockContext["configurationChangeListeners"][number],
    ): { dispose: () => undefined } {
      context.configurationChangeListeners.push(listener);
      return { dispose: (): undefined => undefined };
    },
    onDidChangeTextDocument: disposableEvent,
    onDidChangeWorkspaceFolders: disposableEvent,
    onDidCloseTextDocument: disposableEvent,
    onDidSaveTextDocument: disposableEvent,
    openTextDocument(uri: unknown): {
      getText: () => string;
      languageId: string;
      uri: unknown;
    } {
      return {
        getText: (): string => packageFileFixture("package-left-pad.json"),
        languageId: "json",
        uri,
      };
    },
    workspaceFolders: [{ uri: workspaceUri }],
  };
}

function createVscodeMock(context: VscodeMockContext): MockModule {
  return Object.fromEntries([
    ["CodeLens", createCodeLensConstructor()],
    [
      "EventEmitter",
      createEventEmitterConstructor((): void => {
        context.state.codeLensRefreshCount += 1;
      }),
    ],
    [
      "FileType",
      Object.fromEntries([
        ["Directory", 2],
        ["File", 1],
      ]),
    ],
    ["Range", createCodeLensConstructor()],
    ["RelativePattern", createRelativePatternConstructor()],
    [
      "Uri",
      {
        file: (path: string): { path: string; scheme: string } => ({
          path,
          scheme: "file",
        }),
      },
    ],
    ["WorkspaceEdit", createWorkspaceEditConstructor()],
    ["commands", createCommandsMock(context.commands)],
    [
      "env",
      {
        openExternal(uri: unknown): void {
          context.environment.openedExternalUris.push(uri);
        },
      },
    ],
    ["languages", createLanguagesMock(context.languages)],
    ["tasks", createTasksMock(context.tasks, context.state)],
    ["window", createWindowMock(context.window, context.state)],
    ["workspace", createWorkspaceMock(context.workspace, context.state)],
  ]);
}

export type {
  TaskExecution,
  VscodeMockContext,
} from "./context.ts";
export { createVscodeMock };
