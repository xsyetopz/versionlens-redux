import type { commandState } from "./command-state.ts";

type DependencySnapshotMap = Map<string, string>;

type ConfigurationSubscriptionState = Omit<
  ReturnType<typeof commandState>,
  "ui"
> & {
  context: { extensionPath: string; subscriptions: never[] };
  snapshots: {
    editedDependencies: DependencySnapshotMap;
    savedDependencies: DependencySnapshotMap;
  };
  ui: {
    diagnostics: { delete: () => undefined };
    outputChannel: Record<string, never>;
  };
};

function trackedDependencies(
  label: string,
  cleared: string[],
): DependencySnapshotMap {
  const dependencies = new Map([["file:///package.json", label]]);
  const clearDependencies = dependencies.clear.bind(dependencies);
  dependencies.clear = (): void => {
    cleared.push(label);
    clearDependencies();
  };
  return dependencies;
}

function configurationSubscriptionState(
  context: { extensionPath: string; subscriptions: never[] },
  disposed: string[],
  editedDependencies: DependencySnapshotMap,
  savedDependencies: DependencySnapshotMap,
): ConfigurationSubscriptionState {
  return {
    context,
    flags: {
      codeLensReplace: true,
      providerBusy: 0,
      providerError: false,
      showOutdated: false,
      showPrereleases: true,
      showSuggestionStats: true,
      showVersionLenses: true,
    },
    lifecycle: {
      externalPackageFileWatchers: new Map(),
      packageFileWatchers: [],
      sessionGenerations: new Map(),
    },
    sessions: new Map([
      [
        "workspace:file:///workspace",
        {
          resource: undefined,
          session: {
            disposeSession(): void {
              disposed.push("disposed");
            },
          },
        },
      ],
    ]),
    snapshots: { editedDependencies, savedDependencies },
    ui: {
      diagnostics: { delete: (): undefined => undefined },
      outputChannel: {},
    },
  };
}

export { configurationSubscriptionState, trackedDependencies };
