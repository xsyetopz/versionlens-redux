import type * as vscode from "vscode";

export function configuredValue<T>(
	key: string,
	config: vscode.WorkspaceConfiguration,
) {
	const inspected = config.inspect<T>(key);
	return (
		inspected?.workspaceFolderValue ??
		inspected?.workspaceValue ??
		inspected?.globalValue
	);
}
