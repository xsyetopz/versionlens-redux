import type { WorkspaceConfiguration } from "#vscode-host";

export const configuredValue = (
	key: string,
	config: WorkspaceConfiguration,
): unknown => {
	const inspected = config.inspect<unknown>(key);
	return (
		inspected?.workspaceFolderValue ??
		inspected?.workspaceValue ??
		inspected?.globalValue
	);
};
