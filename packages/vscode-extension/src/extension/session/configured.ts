import type { WorkspaceConfiguration } from "#vscode-host";

export function configuredValue<T>(
  key: string,
  config: WorkspaceConfiguration,
): T | undefined {
  const inspected = config.inspect<T>(key);
  return (
    inspected?.workspaceFolderValue ??
    inspected?.workspaceValue ??
    inspected?.globalValue
  );
}
