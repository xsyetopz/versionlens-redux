import { type Disposable, workspace } from "#vscode-host";
import { restartClient } from "../client.ts";
import { documentSelectors } from "../documents.ts";
import { clearManifestInfo } from "../manifest.ts";
import { reloadConfigurationState } from "../session/flags.ts";
import type { ExtensionState } from "../state.ts";
import { updateContexts } from "./contexts.ts";
import { reportError } from "./error.ts";
import { notifyConfigurationChanged } from "./toggles.ts";

export function registerConfigurationHandler(
  state: ExtensionState,
): Disposable {
  return workspace.onDidChangeConfiguration((event): Promise<void> => {
    if (
      !(
        event.affectsConfiguration("versionlens") ||
        event.affectsConfiguration("http") ||
        event.affectsConfiguration("files.exclude")
      )
    ) {
      return Promise.resolve();
    }
    if (event.affectsConfiguration("versionlens")) {
      reloadConfigurationState(state);
    }
    return synchronizeServer(state);
  });
}

export function registerWorkspaceFolderHandler(
  state: ExtensionState,
): Disposable {
  return workspace.onDidChangeWorkspaceFolders(
    (): Promise<void> => synchronizeServer(state),
  );
}

/**
 * Tells the server to pull the settings again. The documents it is told
 * about come from the settings, so the client starts over when they change.
 */
async function synchronizeServer(state: ExtensionState): Promise<void> {
  try {
    state.snapshots.savedDependencies.clear();
    state.snapshots.editedDependencies.clear();
    clearManifestInfo();
    if (state.selectorKey === JSON.stringify(documentSelectors())) {
      await notifyConfigurationChanged(state);
    } else {
      await restartClient(state);
    }
    await updateContexts(state);
  } catch (error) {
    await reportError(state, "could not apply the settings", error);
  }
}
