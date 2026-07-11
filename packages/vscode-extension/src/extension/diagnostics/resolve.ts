import * as vscode from "vscode";
import { authorizationRequiredMessage } from "../auth/required.ts";
import { addAuthHeader, isAuthHeaderSuppressed } from "../auth.ts";
import { documentInput } from "../documents.ts";
import { invalidateDocumentAnalysis } from "./analyze.ts";
import { recreateSession } from "../session.ts";
import type { ExtensionState } from "../state.ts";
import { logProviderError } from "./log.ts";
import {
	clearProviderError,
	decreaseProviderBusy,
	increaseProviderBusy,
	setProviderError,
} from "./provider.ts";

const addAuthenticationChoice = "Add Authentication";
const pendingAuthenticationDocuments = new Set<string>();

export async function resolveDocumentForDiagnostics(
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
		const output = await state.session.resolveDocument(documentInput(document));
		if (!output) {
			return;
		}
		invalidateDocumentAnalysis(document);
		await offerAuthenticationForDocument(
			state,
			document,
			output.authorizationRequiredCount,
			output.authorizationRequiredRequests?.[0],
		);
	} catch (error) {
		logProviderError(state, error);
		setProviderError(state);
		if (options?.rejectOnError) {
			throw error;
		}
	} finally {
		decreaseProviderBusy(state);
	}
}

async function offerAuthenticationForDocument(
	state: ExtensionState,
	document: vscode.TextDocument,
	count: number,
	authRequest: Parameters<typeof addAuthHeader>[1],
) {
	if (count === 0 || isAuthHeaderSuppressed(state, authRequest)) {
		return;
	}
	const key = document.uri.toString();
	if (pendingAuthenticationDocuments.has(key)) {
		return;
	}
	pendingAuthenticationDocuments.add(key);
	try {
		const choice = await vscode.window.showWarningMessage(
			authorizationRequiredMessage(count),
			{ modal: true },
			addAuthenticationChoice,
		);
		if (choice !== addAuthenticationChoice) {
			return;
		}
		if (!(await addAuthHeader(state, authRequest))) {
			return;
		}
		await recreateSession(state);
		await state.session?.resolveDocument(documentInput(document));
		state.ui.codeLensRefresh?.fire();
	} finally {
		pendingAuthenticationDocuments.delete(key);
	}
}
