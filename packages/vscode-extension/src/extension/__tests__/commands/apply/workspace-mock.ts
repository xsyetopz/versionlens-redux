interface Disposable {
  dispose: () => undefined;
}

interface ConfigurationMock {
  get: (key: string, fallback?: unknown) => unknown;
  inspect: (key: string) => { workspaceValue: unknown | null } | undefined;
  update: (key: string, value: unknown, target: boolean) => void;
}

interface WorkspaceEventMock {
  onDidChangeConfiguration: () => Disposable;
  onDidChangeTextDocument: () => Disposable;
  onDidChangeWorkspaceFolders: () => Disposable;
  onDidCloseTextDocument: () => Disposable;
  onDidSaveTextDocument: () => Disposable;
}

interface WorkspaceOperationsMock {
  applyEdit: (edit: { edits: unknown[] }) => true | Promise<boolean>;
  asRelativePath: (uri: { toString: () => string }) => string;
  findFiles: () => never[];
  getConfiguration: () => ConfigurationMock;
  getWorkspaceFolder: () => undefined;
  openTextDocument: () => undefined;
}

interface WorkspaceMock {
  events: WorkspaceEventMock;
  operations: WorkspaceOperationsMock;
}

interface WorkspaceMockContext {
  appliedEdits: unknown[];
  state: { applyEditBlocker: Promise<boolean> | undefined };
  updatedConfig: { key: string; target: boolean; value: unknown }[];
  workspaceConfig: Record<string, unknown>;
}

function createWorkspaceOperationsMock(
  context: WorkspaceMockContext,
): WorkspaceOperationsMock {
  const { appliedEdits, state, updatedConfig, workspaceConfig } = context;
  return {
    applyEdit: (edit: { edits: unknown[] }): true | Promise<boolean> => {
      appliedEdits.push(...edit.edits);
      if (state.applyEditBlocker === undefined) {
        return true;
      }
      return state.applyEditBlocker;
    },
    asRelativePath: (uri: { toString: () => string }): string => uri.toString(),
    findFiles: (): never[] => [],
    getConfiguration: (): ConfigurationMock => ({
      get(key: string, fallback?: unknown): unknown {
        if (Object.hasOwn(workspaceConfig, key)) {
          return workspaceConfig[key];
        }
        return fallback;
      },
      inspect(key: string): { workspaceValue: unknown | null } | undefined {
        const value = workspaceConfig[key];
        if (value === undefined) {
          return;
        }
        return { workspaceValue: value };
      },
      update(key: string, value: unknown, target: boolean): void {
        workspaceConfig[key] = value;
        updatedConfig.push({ key, target, value });
      },
    }),
    getWorkspaceFolder: (): undefined => undefined,
    openTextDocument: (): undefined => undefined,
  };
}

function createWorkspaceEventsMock(): WorkspaceEventMock {
  const disposable = (): Disposable => ({
    dispose: (): undefined => undefined,
  });
  return {
    onDidChangeConfiguration: disposable,
    onDidChangeTextDocument: disposable,
    onDidChangeWorkspaceFolders: disposable,
    onDidCloseTextDocument: disposable,
    onDidSaveTextDocument: disposable,
  };
}

function createWorkspaceMock(context: WorkspaceMockContext): WorkspaceMock {
  return {
    events: createWorkspaceEventsMock(),
    operations: createWorkspaceOperationsMock(context),
  };
}

export { createWorkspaceMock };
