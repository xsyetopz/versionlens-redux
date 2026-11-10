import { commands, window } from "#vscode-host";

import { fileDocument } from "../documents.ts";
import { manifestInfo } from "../manifest.ts";
import type { ExtensionState } from "../state.ts";
import { customInstallTaskLabel } from "../tasks/custom-install.ts";

export const updateContexts = async (
	state: ExtensionState,
): Promise<string | false> => {
	const activeDocument = fileDocument(window.activeTextEditor?.document);
	const info =
		activeDocument === undefined
			? undefined
			: await manifestInfo(state, activeDocument);
	const provider = info?.supported === true ? info.provider : null;
	const providerActive =
		provider !== null && provider !== "" ? provider : false;
	await setContexts(state, {
		"versionlens.providerActive": providerActive,
		"versionlens.show": state.flags.showVersionLenses,
		"versionlens.showCustomInstall":
			providerActive !== false &&
			Boolean(await customInstallTaskLabel(state, activeDocument)),
		"versionlens.showOutdated": state.flags.showOutdated,
		"versionlens.showPrereleases": state.flags.showPrereleases,
		"versionlens.showSortAlphabetically":
			providerActive !== false && (info?.canSortDependencies ?? false),
	});
	return providerActive;
};

const setContexts = async (
	state: ExtensionState,
	contexts: Record<string, boolean | string>,
): Promise<void> => {
	await Promise.all(
		Object.entries(contexts).map(([key, value]) =>
			commands.executeCommand("setContext", key, value),
		),
	);
	await commands.executeCommand(
		"setContext",
		"versionlens.providerBusy",
		state.flags.providerBusy,
	);
	await commands.executeCommand(
		"setContext",
		"versionlens.providerError",
		state.flags.providerError,
	);
};
