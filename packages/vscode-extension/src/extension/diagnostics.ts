import * as vscode from "vscode";
import { analyzeDocument } from "./diagnostics/analyze.ts";
import { toDiagnostic } from "./diagnostics/convert.ts";
import { setProviderState } from "./diagnostics/provider.ts";
import { resolveDocumentForDiagnostics } from "./diagnostics/resolve.ts";
import {
	dependencySnapshot,
	rememberDependencySnapshot,
} from "./diagnostics/snapshot.ts";
import type { ExtensionState } from "./state.ts";

export { analyzeDocument, dependencySnapshot, setProviderState };

export async function refreshActiveDiagnostics(state: ExtensionState) {
	const document = vscode.window.activeTextEditor?.document;
	if (document) {
		await refreshDiagnostics(state, document);
	}
}

export async function refreshDiagnostics(
	state: ExtensionState,
	document: vscode.TextDocument,
) {
	if (!(state.ui.diagnostics && state.flags.showVersionLenses)) {
		return;
	}

	await resolveDocumentForDiagnostics(state, document);
	const output = analyzeDocument(state, document);
	if (!output) {
		return;
	}
	rememberDependencySnapshot(state, document, output.dependencySignature);
	state.ui.diagnostics.set(document.uri, output.diagnostics.map(toDiagnostic));
}
