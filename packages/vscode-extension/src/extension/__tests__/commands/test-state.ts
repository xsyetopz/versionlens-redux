type AnalyzedDocument =
  | {
      canSortDependencies: boolean;
      status?: { text: string; tooltip: string; visible: boolean };
      activeProviderName?: string;
      installTaskConfigKey?: string;
      isSupportedManifest: boolean;
    }
  | undefined;

type ResolveDiagnosticsHook =
  | ((state: unknown, document: unknown) => unknown)
  | undefined;

interface CommandTestState {
  activeTextEditor: { document: unknown } | undefined;
  activeRefreshCount: number;
  analyzed: AnalyzedDocument;
  codeLensRefreshCount: number;
  dependencyFileType: number;
  dependencySnapshotValue: string;
  refreshCount: number;
  resolveDiagnosticsHook: ResolveDiagnosticsHook;
  taskCompletionMode: "auto" | "manual";
  warningChoice: string | undefined;
}

function createCommandTestState(): CommandTestState {
  return {
    activeTextEditor: undefined,
    activeRefreshCount: 0,
    analyzed: undefined,
    codeLensRefreshCount: 0,
    dependencyFileType: 2,
    dependencySnapshotValue: "",
    refreshCount: 0,
    resolveDiagnosticsHook: undefined,
    taskCompletionMode: "auto",
    warningChoice: undefined,
  };
}

export type { AnalyzedDocument, CommandTestState };
export { createCommandTestState };
