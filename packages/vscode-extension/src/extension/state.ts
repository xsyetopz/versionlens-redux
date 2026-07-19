import type {
  DiagnosticCollection,
  Disposable,
  EventEmitter,
  ExtensionContext,
  OutputChannel,
  Uri,
} from "#vscode-host";
import type { NativeSession } from "./native/module.ts";

export interface ResourceSession {
  resource: Uri | undefined;
  session: NativeSession;
}

export interface ExtensionState {
  context: ExtensionContext | undefined;
  sessions: Map<string, ResourceSession>;
  ui: {
    diagnostics: DiagnosticCollection | undefined;
    outputChannel: OutputChannel | undefined;
    codeLensProvider: Disposable | undefined;
    codeLensRefresh: EventEmitter<void> | undefined;
    resetCodeLensResolutions: (() => void) | undefined;
  };
  flags: {
    showVersionLenses: boolean;
    showOutdated: boolean;
    showPrereleases: boolean;
    showSuggestionStats: boolean;
    providerBusy: number;
    providerError: boolean;
    codeLensReplace: boolean;
  };
  snapshots: {
    savedDependencies: Map<string, string>;
    editedDependencies: Map<string, string>;
  };
  lifecycle: {
    packageFileWatchers: Disposable[];
    externalPackageFileWatchers: Map<string, Disposable[]>;
    sessionGenerations: Map<string, number>;
  };
}

export function createExtensionState(): ExtensionState {
  return {
    context: undefined,
    sessions: new Map(),
    ui: {
      diagnostics: undefined,
      outputChannel: undefined,
      codeLensProvider: undefined,
      codeLensRefresh: undefined,
      resetCodeLensResolutions: undefined,
    },
    flags: {
      showVersionLenses: true,
      showOutdated: false,
      showPrereleases: false,
      showSuggestionStats: false,
      providerBusy: 0,
      providerError: false,
      codeLensReplace: true,
    },
    snapshots: {
      savedDependencies: new Map(),
      editedDependencies: new Map(),
    },
    lifecycle: {
      packageFileWatchers: [],
      externalPackageFileWatchers: new Map(),
      sessionGenerations: new Map(),
    },
  };
}
