import type { CommandTestState } from "../test-state.ts";

type VscodeMockState = CommandTestState;

interface TaskExecution {
  task: { name: string };
  terminate: () => void;
}

interface CommandsMockContext {
  contexts: Record<string, unknown>;
  registeredCommands: Record<string, (...args: unknown[]) => unknown>;
}

interface LanguagesMockContext {
  codeLensRegistrationDisposals: number[];
  codeLensSelectors: unknown[];
}

interface TasksMockContext {
  completeTaskExecution: (
    execution: { task: { name: string } },
    exitCode: number | undefined,
  ) => void;
  executedTasks: string[];
  smokeTaskLabel: string;
  taskEndListeners: ((event: {
    execution: { task: { name: string } };
    exitCode: number | undefined;
  }) => void)[];
  taskExecutions: TaskExecution[];
}

interface WindowMockContext {
  quickPickItems: unknown[];
  quickPickOptions: unknown[];
  shownTextDocuments: unknown[];
  warningMessages: unknown[];
}

interface WorkspaceMockContext {
  appliedEdits: unknown[];
  configurationChangeListeners: ((event: {
    affectsConfiguration: (section: string) => boolean;
  }) => Promise<void> | void)[];
  fileSystemWatchers: { disposed: boolean; pattern: unknown }[];
  findFilesCalls: { exclude?: unknown; include: unknown }[];
  workspaceConfig: Record<string, unknown>;
}

interface EnvironmentMockContext {
  openedExternalUris: unknown[];
}

interface VscodeMockContext {
  commands: CommandsMockContext;
  environment: EnvironmentMockContext;
  languages: LanguagesMockContext;
  state: VscodeMockState;
  tasks: TasksMockContext;
  window: WindowMockContext;
  workspace: WorkspaceMockContext;
}

export type {
  CommandsMockContext,
  LanguagesMockContext,
  TaskExecution,
  TasksMockContext,
  VscodeMockContext,
  VscodeMockState,
  WindowMockContext,
  WorkspaceMockContext,
};
