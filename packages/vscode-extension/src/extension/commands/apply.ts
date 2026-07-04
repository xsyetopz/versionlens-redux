import * as vscode from "vscode";
import { authorizationRequiredMessage } from "../auth/required.ts";
import { addAuthHeader, isAuthHeaderSuppressed } from "../auth.ts";
import { logProviderError } from "../diagnostics/log.ts";
import {
	clearProviderError,
	decreaseProviderBusy,
	increaseProviderBusy,
	setProviderError,
} from "../diagnostics/provider.ts";
import { refreshDiagnostics } from "../diagnostics.ts";
import { documentInput, toRange } from "../documents.ts";
import type { NativeApplyCommand } from "../native/input.ts";
import type {
	NativeTextEdit,
	ResolveDocumentOutput,
} from "../native/output.ts";
import { recreateSession } from "../session.ts";
import type { ExtensionState } from "../state.ts";
import { updateContexts } from "./contexts.ts";

export function registerApplyCommands(
	state: ExtensionState,
	commands: [string, NativeApplyCommand][],
) {
	return commands.map(([commandId, command]) =>
		vscode.commands.registerCommand(commandId, () =>
			applyRustEdits(state, undefined, command, undefined, {
				ignoreCodeLensReplace: command === "sort",
			}),
		),
	);
}

export async function applyRustEdits(
	state: ExtensionState,
	dependencyName?: string,
	command?: NativeApplyCommand,
	selectedVersion?: string,
	options: { ignoreCodeLensReplace?: boolean } = {},
) {
	const editor = vscode.window.activeTextEditor;
	if (
		!(editor && state.session) ||
		(state.flags.codeLensReplace === false && !options.ignoreCodeLensReplace)
	) {
		return;
	}

	let output = applyCommand(
		state,
		editor.document,
		dependencyName,
		command,
		selectedVersion,
	);
	if (!output) {
		return;
	}
	output = await resolveAuthenticationIfNeeded(
		state,
		editor.document,
		output,
		dependencyName,
		command,
		selectedVersion,
	);
	if (!output) {
		return;
	}
	if (output.edits.length === 0) {
		return;
	}
	if (output.vulnerableUpdateCount > 0) {
		const choice = await vscode.window.showWarningMessage(
			vulnerableUpdateMessage(output),
			{ modal: true },
			"Update Anyway",
		);
		if (choice !== "Update Anyway") {
			return;
		}
	}

	await applyTextEdits(
		state,
		editor.document,
		output.edits,
		codeLensReplacementMode(command, dependencyName, selectedVersion),
	);
}

type CodeLensReplacementMode = "disable" | "disableThenEnable" | "preserve";

function codeLensReplacementMode(
	command: NativeApplyCommand | undefined,
	dependencyName: string | undefined,
	selectedVersion: string | undefined,
): CodeLensReplacementMode {
	if (command === "sort") {
		return "preserve";
	}
	if (dependencyName || selectedVersion) {
		return "disable";
	}
	return "disableThenEnable";
}

async function applyTextEdits(
	state: ExtensionState,
	document: vscode.TextDocument,
	edits: NativeTextEdit[],
	replacementMode: CodeLensReplacementMode,
) {
	const workspaceEdit = new vscode.WorkspaceEdit();
	for (const edit of edits) {
		workspaceEdit.replace(document.uri, toRange(edit.range), edit.newText);
	}
	if (replacementMode !== "preserve") {
		state.flags.codeLensReplace = false;
	}
	let applied = false;
	try {
		await vscode.workspace.applyEdit(workspaceEdit);
		applied = true;
		await refreshDiagnostics(state, document);
	} finally {
		if (replacementMode === "disableThenEnable" && applied) {
			state.flags.codeLensReplace = true;
		}
	}
}

async function resolveAuthenticationIfNeeded(
	state: ExtensionState,
	document: vscode.TextDocument,
	output: ResolveDocumentOutput,
	dependencyName: string | undefined,
	command: NativeApplyCommand | undefined,
	selectedVersion: string | undefined,
) {
	if (!(output.authorizationRequiredCount > 0)) {
		return output;
	}
	const authRequest = output.authorizationRequiredRequests?.[0];
	if (isAuthHeaderSuppressed(state, authRequest)) {
		return undefined;
	}
	const choice = await vscode.window.showWarningMessage(
		authorizationRequiredMessage(output.authorizationRequiredCount),
		{ modal: true },
		"Add Authentication",
	);
	if (choice !== "Add Authentication") {
		return undefined;
	}
	const retried = await retryAfterAddingAuthentication(
		state,
		document,
		dependencyName,
		command,
		selectedVersion,
		authRequest,
	);
	return retried && !(retried.authorizationRequiredCount > 0)
		? retried
		: undefined;
}

async function retryAfterAddingAuthentication(
	state: ExtensionState,
	document: vscode.TextDocument,
	dependencyName: string | undefined,
	command: NativeApplyCommand | undefined,
	selectedVersion: string | undefined,
	authRequest: Parameters<typeof addAuthHeader>[1],
) {
	if (!(await addAuthHeader(state, authRequest))) {
		return;
	}
	await reloadAuthBackedSession(state, document);
	return applyCommand(
		state,
		document,
		dependencyName,
		command,
		selectedVersion,
	);
}

async function reloadAuthBackedSession(
	state: ExtensionState,
	document: vscode.TextDocument,
) {
	if (state.context?.extensionPath) {
		await recreateSession(state);
	}
	await updateContexts(state);
	await refreshDiagnostics(state, document);
}

function applyCommand(
	state: ExtensionState,
	document: vscode.TextDocument,
	dependencyName: string | undefined,
	command: NativeApplyCommand | undefined,
	selectedVersion: string | undefined,
) {
	clearProviderError(state);
	increaseProviderBusy(state);
	try {
		return state.session?.applyCommand({
			...(command ? { command } : {}),
			...(dependencyName ? { dependencyName } : {}),
			document: documentInput(document),
			...(selectedVersion ? { selectedVersion } : {}),
		});
	} catch (error) {
		logProviderError(state, error);
		setProviderError(state);
		return;
	} finally {
		decreaseProviderBusy(state);
	}
}

function vulnerableUpdateMessage(output: ResolveDocumentOutput): string {
	if (output.vulnerableUpdatePackage && output.vulnerableUpdateVersion) {
		return `Vulnerabilities found in ${output.vulnerableUpdatePackage}@${output.vulnerableUpdateVersion}. Do you want to continue?`;
	}
	return output.vulnerableUpdateCount === 1
		? "Version Lens found a vulnerability for this dependency. Update anyway?"
		: `Version Lens found vulnerabilities for ${output.vulnerableUpdateCount} dependencies. Update anyway?`;
}
