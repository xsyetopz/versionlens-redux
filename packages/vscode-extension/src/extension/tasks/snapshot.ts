import type { TextDocument } from "#vscode-host";

import { manifestInfo } from "../manifest.ts";
import type { ExtensionState } from "../state.ts";

export const dependencySnapshot = async (
	state: ExtensionState,
	document: TextDocument,
): Promise<string> => {
	const info = await manifestInfo(state, document);
	return info?.dependencySignature ?? "";
};

export const rememberDependencySnapshot = (
	state: ExtensionState,
	document: TextDocument,
	snapshot: string,
): void => {
	const key = document.uri.toString();

	if (document.isDirty) {
		state.snapshots.editedDependencies.set(key, snapshot);
		const saved = state.snapshots.savedDependencies.get(key);
		state.flags.showOutdated = saved !== snapshot;
		if (saved === undefined) {
			state.flags.showOutdated = snapshot !== "";
		}
		return;
	}

	state.snapshots.savedDependencies.set(key, snapshot);
	state.snapshots.editedDependencies.delete(key);
	state.flags.showOutdated = false;
};
