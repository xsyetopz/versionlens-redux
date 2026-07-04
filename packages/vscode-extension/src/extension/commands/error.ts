import * as vscode from "vscode";
import {
	clearProviderBusy,
	clearProviderError,
} from "../diagnostics/provider.ts";
import type { ExtensionState } from "../state.ts";

export function showProviderError(state: ExtensionState) {
	state.ui.outputChannel?.show();
	clearProviderError(state);
	clearProviderBusy(state);
	const document = vscode.window.activeTextEditor?.document;
	if (document) {
		vscode.window.showTextDocument(document);
	}
}
