const workspaceSessionKey = ["workspace", "file:///workspace"].join(":");

interface CommandState {
  lifecycle: {
    externalPackageFileWatchers: Map<unknown, unknown>;
    packageFileWatchers: never[];
    sessionGenerations: Map<unknown, unknown>;
  };
  flags: {
    providerBusy: number;
    providerError: boolean;
    showOutdated: boolean;
    showPrereleases: boolean;
    showSuggestionStats: boolean;
    showVersionLenses: boolean;
    codeLensReplace: boolean;
  };
  sessions: Map<string, { resource: undefined; session: unknown }>;
  ui: {
    codeLensRefresh: undefined;
    diagnostics: undefined;
    outputChannel: undefined;
  };
}

function commandState(
  session: unknown,
  extra: Record<string, unknown> = {},
): CommandState {
  const overrides = extra;
  return {
    lifecycle: {
      externalPackageFileWatchers: new Map(),
      packageFileWatchers: [],
      sessionGenerations: new Map(),
    },
    flags: {
      providerBusy: 0,
      providerError: false,
      showOutdated: false,
      showPrereleases: false,
      showSuggestionStats: false,
      showVersionLenses: true,
      codeLensReplace: true,
    },
    sessions: new Map([
      [workspaceSessionKey, { resource: undefined, session }],
    ]),
    ui: {
      codeLensRefresh: undefined,
      diagnostics: undefined,
      outputChannel: undefined,
    },
    ...overrides,
  };
}

export { commandState, workspaceSessionKey };
