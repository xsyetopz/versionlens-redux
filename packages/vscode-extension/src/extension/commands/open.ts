import * as vscode from "vscode";

export async function openDependency(path: string | undefined) {
	if (!path) {
		return;
	}

	const uri = vscode.Uri.file(path);
	const stat = await vscode.workspace.fs.stat(uri);
	if (stat.type === vscode.FileType.Directory) {
		await vscode.env.openExternal(uri);
		return;
	}

	await vscode.window.showTextDocument(uri);
}
