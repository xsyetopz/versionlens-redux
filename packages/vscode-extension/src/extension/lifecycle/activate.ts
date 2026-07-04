import type * as vscode from "vscode";
import { workspace } from "vscode";
import { updateContexts } from "../commands.ts";
import { refreshActiveDiagnostics } from "../diagnostics.ts";
import { recreateSession, reloadConfigurationState } from "../session.ts";
import type { ExtensionState } from "../state.ts";
import { initializePackageFileWatching } from "./package-watchers.ts";
import { registerExtensionSubscriptions } from "./subscriptions.ts";
import { initializeUi } from "./ui.ts";

export async function activateExtension(
	state: ExtensionState,
	context: vscode.ExtensionContext,
) {
	state.context = context;
	reloadConfigurationState(state);
	initializeUi(state);
	warnWhenCodeLensesDisabled(state);
	registerExtensionSubscriptions(state, context);

	await recreateSession(state);
	await updateContexts(state);
	await initializePackageFileWatching(state);
	await refreshActiveDiagnostics(state);
}

function warnWhenCodeLensesDisabled(state: ExtensionState) {
	const editorCodeLensKey = "editor.codeLens";
	if (
		workspace.getConfiguration().get<boolean>(editorCodeLensKey, true) !== false
	) {
		return;
	}

	state.ui.outputChannel?.appendLine(
		"Code lenses are disabled. This extension won't work unless you enable 'editor.codeLens' in your vscode settings",
	);
}
