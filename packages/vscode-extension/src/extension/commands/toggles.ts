import { recreateSession } from "../session.ts";
import type { ExtensionState } from "../state.ts";
import { refreshCodeLenses } from "./codelens.ts";
import { updateContexts } from "./contexts.ts";

export async function toggleVersionLenses(
	state: ExtensionState,
	next: boolean,
) {
	state.flags.showVersionLenses = next;
	await updateContexts(state);
	refreshCodeLenses(state);
}

export async function togglePrereleases(state: ExtensionState, next: boolean) {
	state.flags.showPrereleases = next;
	await recreateSession(state);
	await updateContexts(state);
	refreshCodeLenses(state);
}
