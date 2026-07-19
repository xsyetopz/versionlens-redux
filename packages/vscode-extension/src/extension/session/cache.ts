import { type Uri, workspace } from "#vscode-host";
import { configuredValue } from "./configured.ts";

export function cacheDurationMinutes(resource?: Uri): number | undefined {
  const config = workspace.getConfiguration("versionlens", resource);
  const minutes = configuredValue<number | null | undefined>(
    "caching.duration",
    config,
  );
  if (minutes === undefined) {
    return;
  }
  return minutes ?? 0;
}
