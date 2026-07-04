import type { ExtensionState } from "../state.ts";

export function logProviderError(state: ExtensionState, error: unknown) {
	console.error(error);
	state.ui.outputChannel?.appendLine(errorText(error));
}

function errorText(error: unknown): string {
	return error instanceof Error
		? (error.stack ?? error.message)
		: String(error);
}
