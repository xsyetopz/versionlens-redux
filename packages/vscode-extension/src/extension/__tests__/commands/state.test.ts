import { expect, it } from "../runtime.ts";

import { commandState } from "./command-state.ts";
import {
  configurationSubscriptionState,
  trackedDependencies,
} from "./configuration-state.ts";
import {
  clearRegisteredCommands,
  codeLensRegistrationDisposals,
  codeLensSelectors,
  configurationChangeListeners,
  contexts,
  createdSessionConfigs,
  fileSystemWatchers,
  registeredCommands,
  shownTextDocuments,
  testState,
} from "./support.ts";

it("updateContexts marks provider active only for supported manifests", async (): Promise<void> => {
  const { updateContexts } = await import("../../commands/contexts.ts");
  testState.activeTextEditor = {
    document: {
      uri: { scheme: "file", toString: (): string => "file:///package.json" },
    },
  };
  testState.analyzed = {
    canSortDependencies: true,
    isSupportedManifest: false,
  };

  await updateContexts({
    flags: { showVersionLenses: true, showPrereleases: false },
  } as never);

  expect(contexts["versionlens.providerActive"]).toBe(false);
  expect(contexts["versionlens.showSortAlphabetically"]).toBe(false);

  testState.analyzed = {
    activeProviderName: "npm",
    canSortDependencies: false,
    isSupportedManifest: true,
  };
  await updateContexts({
    flags: { showVersionLenses: true, showPrereleases: false },
  } as never);

  expect(contexts["versionlens.providerActive"]).toBe("npm");
  expect(contexts["versionlens.showSortAlphabetically"]).toBe(false);

  await updateContexts({
    flags: {
      showOutdated: true,
      showPrereleases: false,
      showVersionLenses: true,
    },
  } as never);

  expect(contexts["versionlens.showOutdated"]).toBe(true);
});

it("updateContexts disables provider actions for non-file active editors", async (): Promise<void> => {
  const { updateContexts } = await import("../../commands/contexts.ts");
  testState.activeTextEditor = {
    document: {
      uri: {
        scheme: "versionlens",
        toString: (): string =>
          "versionlens:/versionlens.multi-registries.json",
      },
    },
  };
  testState.analyzed = {
    activeProviderName: "npm",
    canSortDependencies: true,
    installTaskConfigKey: "npm.onSaveChanges",
    isSupportedManifest: true,
  };

  await updateContexts({
    flags: { showVersionLenses: true, showPrereleases: false },
  } as never);

  expect(contexts["versionlens.providerActive"]).toBe(false);
  expect(contexts["versionlens.showCustomInstall"]).toBe(false);
  expect(contexts["versionlens.showSortAlphabetically"]).toBe(false);
});

it("provider busy context is a numeric counter like upstream", async (): Promise<void> => {
  const { setProviderState } = await import("../../diagnostics/provider.ts");
  const state = commandState(undefined);

  setProviderState(state as never, true, false);
  expect(contexts["versionlens.providerBusy"]).toBe(1);
  setProviderState(state as never, true, false);
  expect(contexts["versionlens.providerBusy"]).toBe(2);
  setProviderState(state as never, false, false);
  expect(contexts["versionlens.providerBusy"]).toBe(1);
  setProviderState(state as never, false, false);
  expect(contexts["versionlens.providerBusy"]).toBe(0);
  setProviderState(state as never, false, false);
  expect(contexts["versionlens.providerBusy"]).toBe(0);
});

it("error click clears all provider busy state like upstream", async (): Promise<void> => {
  const { showProviderError } = await import("../../commands/error.ts");
  testState.activeTextEditor = {
    document: {
      uri: { scheme: "file", toString: (): string => "file:///package.json" },
    },
  };
  const shown: string[] = [];
  const state = commandState(undefined, {
    flags: {
      providerBusy: 3,
      providerError: true,
      showOutdated: false,
      showPrereleases: false,
      showSuggestionStats: false,
      showVersionLenses: true,
      codeLensReplace: true,
    },
    ui: {
      codeLensRefresh: undefined,
      diagnostics: undefined,
      outputChannel: { show: (): number => shown.push("show") },
    },
  });

  showProviderError(state as never);

  expect(shown).toEqual(["show"]);
  expect(state.flags.providerBusy).toBe(0);
  expect(state.flags.providerError).toBe(false);
  expect(contexts["versionlens.providerBusy"]).toBe(0);
  expect(contexts["versionlens.providerError"]).toBe(false);
  expect(shownTextDocuments).toHaveLength(1);
});

it("versionlens configuration changes recreate the native session and refresh active diagnostics", async (): Promise<void> => {
  const { registerExtensionSubscriptions } = await import(
    "../../lifecycle/subscriptions.ts"
  );
  const document = {
    uri: { scheme: "file", toString: (): string => "file:///package.json" },
  };
  const context = { extensionPath: "/extension", subscriptions: [] };
  const disposed: string[] = [];
  configurationChangeListeners.length = 0;
  codeLensRegistrationDisposals.length = 0;
  codeLensSelectors.length = 0;
  createdSessionConfigs.length = 0;
  fileSystemWatchers.length = 0;
  testState.activeRefreshCount = 0;
  testState.codeLensRefreshCount = 0;
  testState.activeTextEditor = { document };
  testState.analyzed = {
    activeProviderName: "npm",
    canSortDependencies: true,
    isSupportedManifest: true,
  };
  const cleared: string[] = [];
  const editedDependencies = trackedDependencies("edited", cleared);
  const savedDependencies = trackedDependencies("saved", cleared);
  const state = configurationSubscriptionState(
    context,
    disposed,
    editedDependencies,
    savedDependencies,
  );

  registerExtensionSubscriptions(state as never, context as never);
  const initialWatcherCount = fileSystemWatchers.length;
  await configurationChangeListeners[0]?.({
    affectsConfiguration: (section: string): boolean =>
      section === "versionlens",
  });

  expect(disposed).toEqual(["disposed"]);
  expect(createdSessionConfigs).toHaveLength(2);
  expect(
    fileSystemWatchers
      .slice(0, initialWatcherCount)
      .every((watcher): boolean => watcher.disposed),
  ).toBe(true);
  expect(fileSystemWatchers.length).toBeGreaterThan(initialWatcherCount);
  expect(new Set(cleared)).toEqual(new Set(["edited", "saved"]));
  expect(testState.codeLensRefreshCount).toBe(1);
  expect(codeLensSelectors).toHaveLength(2);
  expect(codeLensRegistrationDisposals).toHaveLength(1);
  expect(testState.activeRefreshCount).toBe(1);
  expect(contexts["versionlens.providerActive"]).toBe("npm");
});

it("clear cache command clears native cache, diagnostics, and dependency snapshots", async (): Promise<void> => {
  const { registerCommands } = await import("../../commands/register.ts");
  let nativeClearCount = 0;
  let diagnosticsClearCount = 0;
  testState.activeRefreshCount = 0;
  testState.codeLensRefreshCount = 0;
  clearRegisteredCommands();
  const snapshots = {
    editedDependencies: new Map([["file:///package.json", "edited"]]),
    savedDependencies: new Map([["file:///package.json", "saved"]]),
  };
  const state = commandState(
    {
      clearCache: (): void => {
        nativeClearCount += 1;
      },
    },
    {
      snapshots,
      ui: {
        diagnostics: {
          clear: (): void => {
            diagnosticsClearCount += 1;
          },
        },
      },
    },
  );

  registerCommands(state as never);
  await registeredCommands["versionlens.suggestion.onClearCache"]?.();

  expect(nativeClearCount).toBe(1);
  expect(diagnosticsClearCount).toBe(1);
  expect(snapshots.editedDependencies.size).toBe(0);
  expect(snapshots.savedDependencies.size).toBe(0);
  expect(testState.codeLensRefreshCount).toBe(1);
  expect(testState.activeRefreshCount).toBe(1);
});
