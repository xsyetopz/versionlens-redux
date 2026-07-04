import type { ExtensionState } from "../state.ts";
import { disposePackageFileWatchers } from "./package-watchers.ts";
import { disposeUi } from "./ui.ts";

export function deactivateExtension(state: ExtensionState) {
	disposePackageFileWatchers(state);
	disposeUi(state);
	for (const subscription of state.context?.subscriptions ?? []) {
		subscription.dispose();
	}
	state.context?.subscriptions.splice(0);
	state.session?.disposeSession();
	state.session = undefined;
	state.snapshots.savedDependencies.clear();
	state.snapshots.editedDependencies.clear();
	state.context = undefined;
}
