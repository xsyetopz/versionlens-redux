import { window, workspace } from "#vscode-host";

import { fileDocument } from "../documents.ts";
import { manifestInfo } from "../manifest.ts";
import type { ExtensionState } from "../state.ts";
import { runTask } from "./runner.ts";

export const runCustomInstall = async (
	state: ExtensionState,
): Promise<void> => {
	const document = fileDocument(window.activeTextEditor?.document);
	if (!document) {
		return;
	}
	const label = await customInstallTaskLabel(state, document);
	if (label === undefined || label === "") {
		return;
	}

	await runTask(label, workspace.getWorkspaceFolder(document.uri));
};

export const customInstallTaskLabel = async (
	state: ExtensionState,
	document = window.activeTextEditor?.document,
): Promise<string | undefined> => {
	const file = fileDocument(document);
	const info = file === undefined ? undefined : await manifestInfo(state, file);
	const key = info?.installTaskConfigKey;
	return key === undefined || key === null || key === ""
		? undefined
		: workspace.getConfiguration("versionlens").get<string | undefined>(key);
};
