import { mock, mockVscodeHost } from "../runtime.ts";
import {
  appliedEdits as sharedAppliedEdits,
  registeredCommands as sharedRegisteredCommands,
} from "../support.ts";
import { workspaceSessionKey } from "./command-state.ts";
import { type AnalyzedDocument, createCommandTestState } from "./test-state.ts";
import { createVscodeMock, type TaskExecution } from "./vscode/mock.ts";

type MockModule = Record<string, unknown>;

const appliedEdits: unknown[] = sharedAppliedEdits;
const registeredCommands: Record<string, (...args: unknown[]) => unknown> =
  sharedRegisteredCommands;

interface NativeSession {
  analyzeDocument: () => AnalyzedDocument;
  applyCommand: () => undefined;
  clearCache: () => undefined;
  disposeSession: () => undefined;
  resolveDocument: () => undefined;
}

const smokeTaskLabel = "smoke bun install";

function saveTaskState(showOutdated: boolean): {
  flags: { showOutdated: boolean };
  snapshots: {
    editedDependencies: Map<string, string>;
    savedDependencies: Map<string, string>;
  };
} {
  return {
    flags: { showOutdated },
    snapshots: {
      editedDependencies: new Map<string, string>(),
      savedDependencies: new Map<string, string>(),
    },
  };
}

function documentWithUri(uri: string): {
  uri: { scheme: "file"; toString: () => string };
} {
  return { uri: { scheme: "file", toString: (): string => uri } };
}
const contexts: Record<string, unknown> = {};
const executedTasks: string[] = [];
const taskExecutions: TaskExecution[] = [];
const openedExternalUris: unknown[] = [];
const shownTextDocuments: unknown[] = [];
const quickPickItems: unknown[] = [];
const quickPickOptions: unknown[] = [];
const warningMessages: unknown[] = [];
const findFilesCalls: { exclude?: unknown; include: unknown }[] = [];
const fileSystemWatchers: { disposed: boolean; pattern: unknown }[] = [];
const workspaceConfig: Record<string, unknown> = {
  "npm.onSaveChanges": smokeTaskLabel,
};
const createdSessionConfigs: unknown[] = [];
const codeLensRegistrationDisposals: number[] = [];
const codeLensSelectors: unknown[] = [];
const configurationChangeListeners: ((event: {
  affectsConfiguration: (section: string) => boolean;
}) => Promise<void> | void)[] = [];
const taskEndListeners: ((event: {
  execution: { task: { name: string } };
  exitCode: number | undefined;
}) => void)[] = [];
const testState = createCommandTestState();

function clearRegisteredCommands(): void {
  for (const command of Object.keys(registeredCommands)) {
    delete registeredCommands[command];
  }
}

function registeredCommand(name: string): (...args: unknown[]) => unknown {
  const command = registeredCommands[name];
  if (!command) {
    throw new Error(`Command ${name} was not registered`);
  }
  return command;
}

function completeTask(name: string, exitCode: number | undefined): void {
  let execution: TaskExecution | undefined;
  for (let index = taskExecutions.length - 1; index >= 0; index -= 1) {
    const candidate = taskExecutions[index];
    if (candidate?.task.name === name) {
      execution = candidate;
      break;
    }
  }
  if (execution) {
    completeTaskExecution(execution, exitCode);
  }
}

function completeTaskExecution(
  execution: { task: { name: string } },
  exitCode: number | undefined,
): void {
  for (const listener of [...taskEndListeners]) {
    listener({ execution, exitCode });
  }
}

function createNativeSession(): NativeSession {
  return {
    analyzeDocument: (): AnalyzedDocument => testState.analyzed,
    applyCommand: (): undefined => undefined,
    clearCache: (): undefined => undefined,
    disposeSession: (): undefined => undefined,
    resolveDocument: (): undefined => undefined,
  };
}

mockVscodeHost(
  (): MockModule =>
    createVscodeMock({
      commands: { contexts, registeredCommands },
      environment: { openedExternalUris },
      languages: { codeLensRegistrationDisposals, codeLensSelectors },
      state: testState,
      tasks: {
        completeTaskExecution,
        executedTasks,
        smokeTaskLabel,
        taskEndListeners,
        taskExecutions,
      },
      window: {
        quickPickItems,
        quickPickOptions,
        shownTextDocuments,
        warningMessages,
      },
      workspace: {
        appliedEdits,
        configurationChangeListeners,
        fileSystemWatchers,
        findFilesCalls,
        workspaceConfig,
      },
    }),
);

mock.module(
  "../../native.ts",
  (): MockModule => ({
    loadNative: (): MockModule => ({
      createSession(config: unknown): NativeSession {
        createdSessionConfigs.push(config);
        return createNativeSession();
      },
    }),
  }),
);

mock.module(
  "../../diagnostics/refresh.ts",
  (): MockModule => ({
    refreshActiveDiagnostics: (): void => {
      testState.activeRefreshCount += 1;
    },
    refreshDiagnostics: (): void => {
      testState.refreshCount += 1;
    },
  }),
);

mock.module(
  "../../diagnostics/analyze.ts",
  (): MockModule => ({
    analyzeDocument: (): AnalyzedDocument => testState.analyzed,
  }),
);

mock.module(
  "../../diagnostics/snapshot.ts",
  (): MockModule => ({
    dependencySnapshot: (): string => testState.dependencySnapshotValue,
    rememberDependencySnapshot: (): undefined => undefined,
  }),
);

mock.module(
  "../../diagnostics/resolve.ts",
  (): MockModule => ({
    resolveDocumentForDiagnostics: (
      state: {
        sessions?: Map<
          string,
          { session?: { resolveDocument?: (input: unknown) => unknown } }
        >;
      },
      document: unknown,
    ): unknown => {
      if (testState.resolveDiagnosticsHook) {
        return testState.resolveDiagnosticsHook(state, document);
      }
      return state.sessions
        ?.get(workspaceSessionKey)
        ?.session?.resolveDocument?.(document);
    },
  }),
);

export {
  appliedEdits,
  clearRegisteredCommands,
  codeLensRegistrationDisposals,
  codeLensSelectors,
  completeTask,
  configurationChangeListeners,
  contexts,
  createdSessionConfigs,
  documentWithUri,
  executedTasks,
  fileSystemWatchers,
  findFilesCalls,
  openedExternalUris,
  quickPickItems,
  quickPickOptions,
  registeredCommand,
  registeredCommands,
  saveTaskState,
  shownTextDocuments,
  smokeTaskLabel,
  testState,
  warningMessages,
  workspaceConfig,
};
