import { workspace } from "#vscode-host";
import type { ExtensionState } from "../state.ts";
import { configuredValue } from "./configured.ts";

function reloadConfigurationState(state: ExtensionState): void {
  const config = workspace.getConfiguration("versionlens");
  state.flags.showVersionLenses = config.get(
    "suggestions.showOnStartup",
    false,
  );
  state.flags.diagnosticsHidden = false;
  state.flags.showPrereleases = prereleasesOnStartup();
}

function prereleasesOnStartup(): boolean {
  return (
    configuredValue<boolean>(
      "suggestions.showPrereleasesOnStartup",
      workspace.getConfiguration("versionlens"),
    ) ?? false
  );
}

export { reloadConfigurationState };
