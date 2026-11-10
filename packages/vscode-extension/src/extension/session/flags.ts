import { workspace } from "#vscode-host";

import type { ExtensionState } from "../state.ts";
import { configuredValue } from "./configured.ts";

const reloadConfigurationState = (state: ExtensionState): void => {
	const config = workspace.getConfiguration("versionlens");
	state.flags.showVersionLenses = config.get(
		"suggestions.showOnStartup",
		false,
	);
	state.flags.diagnosticsHidden = false;
	state.flags.showPrereleases = prereleasesOnStartup();
};

const prereleasesOnStartup = (): boolean =>
	configuredValue(
		"suggestions.showPrereleasesOnStartup",
		workspace.getConfiguration("versionlens"),
	) === true;

export { reloadConfigurationState };
