import * as vscode from "vscode";
import type { ExtensionState } from "../state.ts";
import { configuredValue } from "./configured.ts";

export function configuredEnabledProviders() {
	return configuredValue<string[] | undefined>(
		"enabledProviders",
		vscode.workspace.getConfiguration("versionlens"),
	);
}

export function configuredShowVulnerabilities() {
	return configuredValue<boolean | undefined>(
		"suggestions.showVulnerabilities",
		vscode.workspace.getConfiguration("versionlens"),
	);
}

export function reloadConfigurationState(state: ExtensionState) {
	const config = vscode.workspace.getConfiguration("versionlens");
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

function prereleasesOnStartup() {
	return (
		configuredValue<boolean>(
			"suggestions.showPrereleasesOnStartup",
			vscode.workspace.getConfiguration("versionlens"),
		) ?? false
	);
}
