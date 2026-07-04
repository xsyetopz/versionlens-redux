import * as vscode from "vscode";
import { registerCommands, updateContexts } from "../commands.ts";
import { analyzeDocument, refreshDiagnostics } from "../diagnostics.ts";
import { fileDocument } from "../documents.ts";
import type { ExtensionState } from "../state.ts";
import { handleDidSaveTextDocument } from "../tasks.ts";
import {
	registerPackageFileWatchers,
	watchActivePackageFileOutsideWorkspace,
} from "./package-watchers.ts";
import { registerRefreshTimer } from "./refresh-timer.ts";

export function registerExtensionSubscriptions(
	state: ExtensionState,
	context: vscode.ExtensionContext,
) {
	const diagnostics = state.ui.diagnostics;
	const outputChannel = state.ui.outputChannel;
	if (!(diagnostics && outputChannel)) {
		return;
	}

	const packageFileWatchers = registerPackageFileWatchers(state);

	context.subscriptions.push(
		diagnostics,
		outputChannel,
		...registerCommands(state),
		...packageFileWatchers,
		registerRefreshTimer(state),
		vscode.workspace.onDidChangeTextDocument(async (event) => {
			if (!isRelevantTextDocumentChange(event)) {
				return;
			}
			const output = analyzeDocument(state, event.document);
			if (!output?.isSupportedManifest) {
				return;
			}

			await refreshDiagnostics(state, event.document);
			if (event.document === vscode.window.activeTextEditor?.document) {
				await updateContexts(state);
			}
		}),
		vscode.workspace.onDidSaveTextDocument((document) =>
			handleDidSaveTextDocument(state, document),
		),
		vscode.workspace.onDidCloseTextDocument((closedDocument) => {
			const document = fileDocument(closedDocument);
			if (!document) {
				return;
			}

			const output = analyzeDocument(state, document);
			if (!output?.isSupportedManifest) {
				return;
			}

			state.snapshots.editedDependencies.delete(document.uri.toString());
		}),
		vscode.window.onDidChangeActiveTextEditor(async (editor) => {
			const providerActive = await updateContexts(state);
			const document = fileDocument(editor?.document);
			if (document && providerActive) {
				watchActivePackageFileOutsideWorkspace(state, document);
			}
		}),
	);
}

function isRelevantTextDocumentChange(event: vscode.TextDocumentChangeEvent) {
	return (
		event.reason === vscode.TextDocumentChangeReason.Undo ||
		event.reason === vscode.TextDocumentChangeReason.Redo ||
		event.contentChanges.length > 0
	);
}
