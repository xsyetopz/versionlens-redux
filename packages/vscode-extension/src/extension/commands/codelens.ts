import * as vscode from "vscode";
import { resolveDocumentForDiagnostics } from "../diagnostics/resolve.ts";
import { analyzeDocument } from "../diagnostics.ts";
import { documentSelectors, toRange } from "../documents.ts";
import type { NativeCodeLensPayload } from "../native/output.ts";
import type { ExtensionState } from "../state.ts";

const nativeArgumentsByCodeLens = new WeakMap<object, string[]>();
const pendingCodeLensResolutions = new Set<string>();
const completedCodeLensResolutions = new Set<string>();

export function registerCodeLensProvider(state: ExtensionState) {
	state.ui.codeLensRefresh?.dispose();
	pendingCodeLensResolutions.clear();
	completedCodeLensResolutions.clear();
	const refresh = new vscode.EventEmitter<void>();
	state.ui.codeLensRefresh = refresh;
	const registration = vscode.languages.registerCodeLensProvider(
		documentSelectors(),
		{
			onDidChangeCodeLenses: refresh.event,
			provideCodeLenses(document) {
				if (!state.flags.showVersionLenses) {
					return [];
				}

				const output = analyzeDocument(state, document, {
					rejectOnError: true,
				});
				if (output) {
					scheduleCodeLensResolution(
						state,
						document,
						output.dependencySignature,
					);
				}
				state.flags.codeLensReplace = true;
				return (output?.codeLenses ?? []).map(toCodeLens);
			},
		},
	);
	return {
		dispose() {
			registration.dispose();
			refresh.dispose();
			if (state.ui.codeLensRefresh === refresh) {
				state.ui.codeLensRefresh = undefined;
			}
		},
	};
}

function scheduleCodeLensResolution(
	state: ExtensionState,
	document: vscode.TextDocument,
	dependencySignature: string,
) {
	const key = `${document.uri.toString()}\0${dependencySignature}`;
	if (
		dependencySignature === "" ||
		pendingCodeLensResolutions.has(key) ||
		completedCodeLensResolutions.has(key)
	) {
		return;
	}

	pendingCodeLensResolutions.add(key);
	setTimeout(() => {
		if (!state.flags.showVersionLenses) {
			pendingCodeLensResolutions.delete(key);
			return;
		}
		resolveDocumentForDiagnostics(state, document)
			.catch(() => undefined)
			.finally(() => {
				pendingCodeLensResolutions.delete(key);
				completedCodeLensResolutions.add(key);
				if (state.flags.showVersionLenses) {
					state.ui.codeLensRefresh?.fire();
				}
			});
	}, 0);
}

export function nativeCodeLensArguments(argument: unknown) {
	if (typeof argument !== "object" || argument === null) {
		return undefined;
	}

	return nativeArgumentsByCodeLens.get(argument);
}

function toCodeLens(lens: NativeCodeLensPayload) {
	const rendered = new vscode.CodeLens(toRange(lens.range));
	nativeArgumentsByCodeLens.set(rendered, lens.arguments);
	rendered.command = {
		command: lens.command,
		title: lens.title,
	};
	if (lens.command) {
		rendered.command.arguments = [rendered];
	}
	return rendered;
}

export function refreshCodeLenses(state: ExtensionState) {
	completedCodeLensResolutions.clear();
	state.ui.codeLensRefresh?.fire();
}
