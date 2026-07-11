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

type CachedAnalysis = {
	session: NonNullable<ExtensionState["session"]>;
	version: number;
	output: ReturnType<NonNullable<ExtensionState["session"]>["analyzeDocument"]>;
};

const analysisCache = new Map<string, CachedAnalysis>();
const maximumCachedAnalyses = 128;

export function invalidateDocumentAnalysis(document: vscode.TextDocument) {
	analysisCache.delete(document.uri.toString());
}

export function analyzeDocument(
	state: ExtensionState,
	document: vscode.TextDocument,
	options?: { rejectOnError?: boolean },
) {
	if (!state.session) {
		return;
	}

	const cacheKey = document.uri.toString();
	const cached = analysisCache.get(cacheKey);
	if (
		cached?.session === state.session &&
		cached.version === document.version
	) {
		return cached.output;
	}

	clearProviderError(state);
	increaseProviderBusy(state);
	try {
		const output = state.session.analyzeDocument(documentInput(document));
		if (analysisCache.size >= maximumCachedAnalyses) {
			analysisCache.clear();
		}
		analysisCache.set(cacheKey, {
			session: state.session,
			version: document.version,
			output,
		});
		return output;
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
