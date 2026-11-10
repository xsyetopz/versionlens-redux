import { window } from "#vscode-host";

import type { ExtensionState } from "../state.ts";

export const initializeUi = (state: ExtensionState): void => {
	state.ui.outputChannel = window.createOutputChannel("Version Lens", {
		log: true,
	});
};

export const disposeUi = (state: ExtensionState): void => {
	state.ui.outputChannel?.dispose();
	state.ui.outputChannel = undefined;
};
