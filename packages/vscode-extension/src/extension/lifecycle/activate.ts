import type * as vscode from "vscode";
import { commands, extensions, window, workspace } from "vscode";
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
	await warnWhenOriginalVersionLensInstalled(state);
	warnWhenCodeLensesDisabled(state);
	registerExtensionSubscriptions(state, context);

	await recreateSession(state);
	await updateContexts(state);
	await initializePackageFileWatching(state);
	await refreshActiveDiagnostics(state);
}

async function warnWhenOriginalVersionLensInstalled(state: ExtensionState) {
	const originalExtensionId = "pflannery.vscode-versionlens";
	if (!extensions.getExtension(originalExtensionId)) {
		return;
	}

	const disableAction = "Disable original VersionLens";
	const message =
		"VersionLens Redux conflicts with the original VersionLens extension. Disable the original extension before using VersionLens Redux in this workspace.";
	state.ui.outputChannel?.appendLine(message);

	const selected = await window.showWarningMessage(message, disableAction);
	if (selected !== disableAction) {
		return;
	}

	await commands.executeCommand(
		"workbench.extensions.action.disableExtension",
		originalExtensionId,
	);
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
