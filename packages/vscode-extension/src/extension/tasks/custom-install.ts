import * as vscode from "vscode";
import { analyzeDocument } from "../diagnostics.ts";
import { activeFileDocument, fileDocument } from "../documents.ts";
import type { ExtensionState } from "../state.ts";
import { runTask } from "./runner.ts";

export async function runCustomInstall(state: ExtensionState) {
	const document = activeFileDocument();
	const label = document ? customInstallTaskLabel(state, document) : undefined;
	if (!label) {
		return;
	}

	await runTask(label);
}

export function customInstallTaskLabel(
	state: ExtensionState,
	document = vscode.window.activeTextEditor?.document,
) {
	const file = fileDocument(document);
	const key = file
		? analyzeDocument(state, file)?.installTaskConfigKey
		: undefined;
	return key
		? vscode.workspace
				.getConfiguration("versionlens")
				.get<string | undefined>(key)
		: undefined;
}
