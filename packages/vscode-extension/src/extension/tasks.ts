import type * as vscode from "vscode";
import { analyzeDocument, refreshDiagnostics } from "./diagnostics.ts";
import { fileDocument } from "./documents.ts";
import type { ExtensionState } from "./state.ts";
import {
	customInstallTaskLabel,
	runCustomInstall,
} from "./tasks/custom-install.ts";
import { runInstallTaskIfDependenciesChanged } from "./tasks/save.ts";

export { customInstallTaskLabel, runCustomInstall };

export async function handleDidSaveTextDocument(
	state: ExtensionState,
	document: vscode.TextDocument,
) {
	const file = fileDocument(document);
	if (!file) {
		return;
	}

	const output = analyzeDocument(state, file);
	if (!output?.isSupportedManifest) {
		return;
	}

	const saved = await runInstallTaskIfDependenciesChanged(state, file);
	if (saved) {
		await refreshDiagnostics(state, file);
	}
}
