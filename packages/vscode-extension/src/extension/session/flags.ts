import { type Uri, workspace } from "#vscode-host";
import type { ExtensionState } from "../state.ts";
import { configuredValue } from "./configured.ts";

function configuredEnabledProviders(resource?: Uri): string[] | undefined {
  return configuredValue<string[] | undefined>(
    "enabledProviders",
    workspace.getConfiguration("versionlens", resource),
  );
}

function configuredShowVulnerabilities(resource?: Uri): boolean | undefined {
  return configuredValue<boolean | undefined>(
    "suggestions.showVulnerabilities",
    workspace.getConfiguration("versionlens", resource),
  );
}

function reloadConfigurationState(state: ExtensionState): void {
  const config = workspace.getConfiguration("versionlens");
  state.flags.showVersionLenses = config.get(
    "suggestions.showOnStartup",
    false,
  );
  state.flags.showPrereleases = prereleasesOnStartup();
  state.flags.showSuggestionStats = config.get(
    "suggestions.showSuggestionsStats",
    false,
  );
}

function prereleasesOnStartup(): boolean {
  return (
    configuredValue<boolean>(
      "suggestions.showPrereleasesOnStartup",
      workspace.getConfiguration("versionlens"),
    ) ?? false
  );
}

export {
  configuredEnabledProviders,
  configuredShowVulnerabilities,
  reloadConfigurationState,
};
