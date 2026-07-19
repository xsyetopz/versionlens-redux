import { type Disposable, workspace } from "#vscode-host";
import { refreshActiveDiagnostics } from "../diagnostics/refresh.ts";
import {
  disposePackageFileWatchers,
  initializePackageFileWatching,
  registerPackageFileWatchers,
} from "../lifecycle/package-watchers.ts";
import { reloadConfigurationState } from "../session/flags.ts";
import {
  recreateAffectedSessions,
  synchronizeWorkspaceSessions,
} from "../session/registry.ts";
import type { ExtensionState } from "../state.ts";
import { refreshCodeLenses, registerCodeLensProvider } from "./codelens.ts";
import { updateContexts } from "./contexts.ts";

export function registerConfigurationHandler(
  state: ExtensionState,
): Disposable {
  return workspace.onDidChangeConfiguration(async (event): Promise<void> => {
    if (
      event.affectsConfiguration("versionlens") ||
      event.affectsConfiguration("http")
    ) {
      if (event.affectsConfiguration("versionlens")) {
        reloadConfigurationState(state);
      }
      state.snapshots.savedDependencies.clear();
      state.snapshots.editedDependencies.clear();
      if (!(await recreateAffectedSessions(state, event))) {
        return;
      }
      if (event.affectsConfiguration("versionlens")) {
        state.context?.subscriptions.push(registerCodeLensProvider(state));
        disposePackageFileWatchers(state);
        state.context?.subscriptions.push(
          ...registerPackageFileWatchers(state),
        );
        await initializePackageFileWatching(state);
      }
      await updateContexts(state);
      refreshCodeLenses(state);
      await refreshActiveDiagnostics(state);
    }
  });
}

export function registerWorkspaceFolderHandler(
  state: ExtensionState,
): Disposable {
  return workspace.onDidChangeWorkspaceFolders(async (): Promise<void> => {
    state.snapshots.savedDependencies.clear();
    state.snapshots.editedDependencies.clear();
    if (!(await synchronizeWorkspaceSessions(state))) {
      return;
    }
    state.context?.subscriptions.push(registerCodeLensProvider(state));
    disposePackageFileWatchers(state);
    state.context?.subscriptions.push(...registerPackageFileWatchers(state));
    await initializePackageFileWatching(state);
    await updateContexts(state);
    refreshCodeLenses(state);
    await refreshActiveDiagnostics(state);
  });
}
