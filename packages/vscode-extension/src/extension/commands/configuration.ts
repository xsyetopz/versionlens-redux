import * as vscode from "vscode";
import { refreshActiveDiagnostics } from "../diagnostics.ts";
import {
	disposePackageFileWatchers,
	initializePackageFileWatching,
	registerPackageFileWatchers,
} from "../lifecycle/package-watchers.ts";
import { recreateSession, reloadConfigurationState } from "../session.ts";
import type { ExtensionState } from "../state.ts";
import { refreshCodeLenses } from "./codelens.ts";
import { updateContexts } from "./contexts.ts";

export function registerConfigurationHandler(state: ExtensionState) {
	return vscode.workspace.onDidChangeConfiguration(async (event) => {
		if (
			event.affectsConfiguration("versionlens") ||
			event.affectsConfiguration("http")
		) {
			if (event.affectsConfiguration("versionlens")) {
				reloadConfigurationState(state);
			}
			state.snapshots.savedDependencies.clear();
			state.snapshots.editedDependencies.clear();
			await recreateSession(state);
			if (event.affectsConfiguration("versionlens")) {
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
