import type { LanguageClient } from "vscode-languageclient/node";
import type { ExtensionContext, LogOutputChannel } from "#vscode-host";

export type DocumentActivity = "busy" | "error";

export interface ExtensionState {
  context: ExtensionContext | undefined;
  client: LanguageClient | undefined;
  /** Tail of the chain that serializes server start, stop and restart. */
  lifecycle: Promise<void>;
  /** Serialized document selector the running client was created with. */
  selectorKey: string | undefined;
  ui: {
    outputChannel: LogOutputChannel | undefined;
  };
  flags: {
    showVersionLenses: boolean;
    showOutdated: boolean;
    showPrereleases: boolean;
    providerBusy: number;
    providerError: boolean;
  };
  /** Busy and error documents reported by `versionlens/status`. */
  activity: Map<string, DocumentActivity>;
  /** Hosts the user was already asked about in this session. */
  promptedHosts: Set<string>;
  snapshots: {
    savedDependencies: Map<string, string>;
    editedDependencies: Map<string, string>;
  };
}

export function createExtensionState(): ExtensionState {
  return {
    context: undefined,
    client: undefined,
    lifecycle: Promise.resolve(),
    selectorKey: undefined,
    ui: {
      outputChannel: undefined,
    },
    flags: {
      showVersionLenses: true,
      showOutdated: false,
      showPrereleases: false,
      providerBusy: 0,
      providerError: false,
    },
    activity: new Map(),
    promptedHosts: new Set(),
    snapshots: {
      savedDependencies: new Map(),
      editedDependencies: new Map(),
    },
  };
}
