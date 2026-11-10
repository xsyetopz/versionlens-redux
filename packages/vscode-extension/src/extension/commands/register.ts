import { commands, type Disposable } from "#vscode-host";

import { removeAuthHeader } from "../auth/remove.ts";
import { addAuthHeader } from "../auth/set.ts";
import { restartClient, startClient } from "../client.ts";
import type { ExtensionState } from "../state.ts";
import { runCustomInstall } from "../tasks/custom-install.ts";
import { updateContexts } from "./contexts.ts";
import { reportError, showProviderError } from "./error.ts";
import {
	runDocumentCommand,
	runServerCommand,
	SERVER_COMMANDS,
	type UpdateLevel,
} from "./server.ts";
import { togglePrereleases, toggleVersionLenses } from "./toggles.ts";

type Handler = () => Promise<void>;
type Register = (id: string, handler: Handler) => Disposable;

/**
 * Registers a command that shows its failures to the user. Commands the
 * server implements are forwarded with `workspace/executeCommand`. The
 * others (toggles, authentication, tasks) run in the extension.
 */
const registerCommand = (
	state: ExtensionState,
	id: string,
	handler: Handler,
): Disposable =>
	commands.registerCommand(id, async (): Promise<void> => {
		try {
			await handler();
		} catch (error) {
			await reportError(state, `${id} failed`, error);
		}
	});

const registerToggleCommands = (
	state: ExtensionState,
	register: Register,
): Disposable[] => [
	register("versionlens.editor.onShowVersionLenses", () =>
		toggleVersionLenses(state, true),
	),
	register("versionlens.editor.onHideVersionLenses", () =>
		toggleVersionLenses(state, false),
	),
	register("versionlens.editor.onShowError", () => showProviderError(state)),
	register("versionlens.icons.showingProgress", () => Promise.resolve()),
	register("versionlens.editor.onShowPrereleaseVersions", () =>
		togglePrereleases(state, true),
	),
	register("versionlens.editor.onHidePrereleaseVersions", () =>
		togglePrereleases(state, false),
	),
];

const registerUpdateCommands = (
	state: ExtensionState,
	register: Register,
): Disposable[] => {
	const updateAll = (level: UpdateLevel) => async (): Promise<void> => {
		await runDocumentCommand(state, SERVER_COMMANDS.updateAll, { level });
	};
	const updateWorkspace = (level: UpdateLevel) => async (): Promise<void> => {
		await startClient(state, true);
		await runServerCommand(state, SERVER_COMMANDS.updateWorkspace, { level });
	};

	return [
		register(
			"versionlens.editor.onUpdateDependenciesLatest",
			updateAll("latest"),
		),
		register(
			"versionlens.editor.onUpdateDependenciesMajor",
			updateAll("major"),
		),
		register(
			"versionlens.editor.onUpdateDependenciesMinor",
			updateAll("minor"),
		),
		register(
			"versionlens.editor.onUpdateDependenciesPatch",
			updateAll("patch"),
		),
		register(
			"versionlens.workspace.onUpdateDependenciesLatest",
			updateWorkspace("latest"),
		),
		register(
			"versionlens.workspace.onUpdateDependenciesMajor",
			updateWorkspace("major"),
		),
		register(
			"versionlens.workspace.onUpdateDependenciesMinor",
			updateWorkspace("minor"),
		),
		register(
			"versionlens.workspace.onUpdateDependenciesPatch",
			updateWorkspace("patch"),
		),
	];
};

const registerCommands = (state: ExtensionState): Disposable[] => {
	const register: Register = (id, handler): Disposable =>
		registerCommand(state, id, handler);

	return [
		...registerToggleCommands(state, register),
		register("versionlens.suggestion.onClearCache", async () => {
			state.snapshots.savedDependencies.clear();
			state.snapshots.editedDependencies.clear();
			await runServerCommand(state, SERVER_COMMANDS.clearCache);
		}),
		register("versionlens.editor.onSortDependencies", async () => {
			await runDocumentCommand(state, SERVER_COMMANDS.sort);
		}),
		...registerUpdateCommands(state, register),
		register("versionlens.server.onRestart", async () => {
			await restartClient(state, true);
			await updateContexts(state);
		}),
		register("versionlens.server.onShowOutput", () => {
			state.ui.outputChannel?.show();
			return Promise.resolve();
		}),
		register("versionlens.onRestartExtensionHost", async () => {
			await commands.executeCommand("workbench.action.restartExtensionHost");
		}),
		register("versionlens.editor.onAddUrlAuthentication", async () => {
			if (await addAuthHeader(state)) {
				await reloadCredentials(state);
			}
		}),
		register("versionlens.editor.onRemoveUrlAuthentication", async () => {
			if (await removeAuthHeader(state)) {
				await reloadCredentials(state);
			}
		}),
		register("versionlens.editor.onCustomInstall", () =>
			runCustomInstall(state),
		),
	];
};

/**
 * The server asks for a credential once per host and session, so a new
 * session is how it learns about a changed one.
 */
const reloadCredentials = async (state: ExtensionState): Promise<void> => {
	await restartClient(state);
	await updateContexts(state);
};

export { registerCommands };
