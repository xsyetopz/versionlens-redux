import type * as vscode from "vscode";
import { documentInput } from "../documents.ts";
import type { ExtensionState } from "../state.ts";
import { logProviderError } from "./log.ts";
import {
	clearProviderError,
	decreaseProviderBusy,
	increaseProviderBusy,
	setProviderError,
} from "./provider.ts";

export function analyzeDocument(
	state: ExtensionState,
	document: vscode.TextDocument,
	options?: { rejectOnError?: boolean },
) {
	if (!state.session) {
		return;
	}

	clearProviderError(state);
	increaseProviderBusy(state);
	try {
		return state.session.analyzeDocument(documentInput(document));
	} catch (error) {
		logProviderError(state, error);
		setProviderError(state);
		if (options?.rejectOnError) {
			throw error;
		}
		return;
	} finally {
		decreaseProviderBusy(state);
	}
}
