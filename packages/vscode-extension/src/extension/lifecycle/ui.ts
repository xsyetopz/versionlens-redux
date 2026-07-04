import * as vscode from "vscode";
import type { ExtensionState } from "../state.ts";

export function initializeUi(state: ExtensionState) {
	state.ui.diagnostics = vscode.languages.createDiagnosticCollection(
		"versionlens-vulnerabilities",
	);
	state.ui.outputChannel = vscode.window.createOutputChannel("Version Lens");
}

export function disposeUi(state: ExtensionState) {
	state.ui.diagnostics?.clear();
	state.ui.diagnostics = undefined;
	state.ui.outputChannel?.dispose();
	state.ui.outputChannel = undefined;
	state.ui.codeLensRefresh?.dispose();
	state.ui.codeLensRefresh = undefined;
}
