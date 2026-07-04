import * as vscode from "vscode";

export function activeFileDocument() {
	return fileDocument(vscode.window.activeTextEditor?.document);
}

export function fileDocument(document: vscode.TextDocument | undefined) {
	return document?.uri.scheme === "file" ? document : undefined;
}
