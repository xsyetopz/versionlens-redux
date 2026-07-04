import type * as vscode from "vscode";
import type { ExtensionState } from "../state.ts";
import { analyzeDocument } from "./analyze.ts";

export function dependencySnapshot(
	state: ExtensionState,
	document: vscode.TextDocument,
) {
	return analyzeDocument(state, document)?.dependencySignature ?? "";
}

export function rememberDependencySnapshot(
	state: ExtensionState,
	document: vscode.TextDocument,
	snapshot: string,
) {
	const key = document.uri.toString();

	if (document.isDirty) {
		state.snapshots.editedDependencies.set(key, snapshot);
		const saved = state.snapshots.savedDependencies.get(key);
		state.flags.showOutdated =
			saved === undefined ? snapshot !== "" : saved !== snapshot;
		return;
	}

	state.snapshots.savedDependencies.set(key, snapshot);
	state.snapshots.editedDependencies.delete(key);
	state.flags.showOutdated = false;
}
