import { type Disposable, workspace } from "#vscode-host";

import {
	applyDiagnosticsVisibility,
	restartClient,
	startClient,
} from "../client.ts";
import { documentSelectors } from "../documents.ts";
import { clearManifestInfo } from "../manifest.ts";
import { reloadConfigurationState } from "../session/flags.ts";
import type { ExtensionState } from "../state.ts";
import { updateContexts } from "./contexts.ts";
import { reportError } from "./error.ts";
import { notifyConfigurationChanged } from "./toggles.ts";

export const registerConfigurationHandler = (
	state: ExtensionState,
): Disposable =>
	workspace.onDidChangeConfiguration((event): Promise<void> => {
		if (!(
			event.affectsConfiguration("versionlens") ||
			event.affectsConfiguration("http") ||
			event.affectsConfiguration("files.exclude")
		)) {
			return Promise.resolve();
		}
		if (event.affectsConfiguration("versionlens")) {
			reloadConfigurationState(state);
			applyDiagnosticsVisibility(state);
		}
		return synchronizeServer(state);
	});

export const registerWorkspaceFolderHandler = (
	state: ExtensionState,
): Disposable =>
	workspace.onDidChangeWorkspaceFolders((): Promise<void> =>
		synchronizeServer(state),
	);

/**
 * Tells the server to pull the settings again. The documents it is told
 * about come from the settings, so the client starts over when they change.
 */
const synchronizeServer = async (state: ExtensionState): Promise<void> => {
	try {
		state.snapshots.savedDependencies.clear();
		state.snapshots.editedDependencies.clear();
		clearManifestInfo();
		if (!state.client) {
			await startClient(state);
		} else if (state.selectorKey === JSON.stringify(documentSelectors())) {
			await notifyConfigurationChanged(state);
		} else {
			await restartClient(state);
		}
		await updateContexts(state);
	} catch (error) {
		await reportError(state, "could not apply the settings", error);
	}
};
