import * as vscode from "vscode";
import type { NativeDocumentInput } from "../native/input.ts";

export function documentInput(
	document: vscode.TextDocument,
): NativeDocumentInput {
	const workspaceRoot = vscode.workspace.getWorkspaceFolder(document.uri)?.uri
		.fsPath;

	return {
		uri: document.uri.toString(),
		languageId: document.languageId,
		text: document.getText(),
		...(workspaceRoot ? { workspaceRoot } : {}),
	};
}
