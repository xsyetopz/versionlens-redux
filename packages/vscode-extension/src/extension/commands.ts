import * as vscode from "vscode";
import { addAuthHeader, removeAuthHeader } from "./auth.ts";
import { applyRustEdits, registerApplyCommands } from "./commands/apply.ts";
import { chooseBuild } from "./commands/build.ts";
import {
	nativeCodeLensArguments,
	refreshCodeLenses,
	registerCodeLensProvider,
} from "./commands/codelens.ts";
import { registerConfigurationHandler } from "./commands/configuration.ts";
import { updateContexts } from "./commands/contexts.ts";
import { showProviderError } from "./commands/error.ts";
import { openDependency } from "./commands/open.ts";
import { togglePrereleases, toggleVersionLenses } from "./commands/toggles.ts";
import { refreshActiveDiagnostics } from "./diagnostics.ts";
import type { NativeApplyCommand } from "./native/input.ts";
import { recreateSession } from "./session.ts";
import type { ExtensionState } from "./state.ts";
import { runCustomInstall } from "./tasks.ts";

export { updateContexts };

export function registerCommands(state: ExtensionState): vscode.Disposable[] {
	const clearCache = async () => {
		state.session?.clearCache();
		state.snapshots.savedDependencies.clear();
		state.snapshots.editedDependencies.clear();
		state.ui.diagnostics?.clear();
		refreshCodeLenses(state);
		await refreshActiveDiagnostics(state);
	};

	return [
		vscode.commands.registerCommand(
			"versionlens.suggestion.onClearCache",
			clearCache,
		),
		vscode.commands.registerCommand(
			"versionlens.suggestion.onUpdateDependency",
			(
				codeLensOrName?: unknown,
				selector?: string,
				command?: NativeApplyCommand,
				selectedVersion?: string,
			) => {
				const nativeArguments = nativeCodeLensArguments(codeLensOrName);
				if (nativeArguments) {
					return applyRustEdits(
						state,
						nativeArguments[1] ?? nativeArguments[0],
						nativeArguments[2] as NativeApplyCommand | undefined,
						nativeArguments[3],
					);
				}

				const name =
					typeof codeLensOrName === "string" ? codeLensOrName : undefined;
				return applyRustEdits(
					state,
					selector ?? name,
					command,
					selectedVersion,
				);
			},
		),
		vscode.commands.registerCommand(
			"versionlens.suggestion.onFileLink",
			(codeLensOrPath?: unknown) => {
				const nativeArguments = nativeCodeLensArguments(codeLensOrPath);
				const path = nativeArguments?.[0] ?? codeLensOrPath;
				return openDependency(typeof path === "string" ? path : undefined);
			},
		),
		vscode.commands.registerCommand(
			"versionlens.suggestion.onChooseBuild",
			(codeLensOrSelector?: unknown, ...builds: string[]) => {
				const nativeArguments = nativeCodeLensArguments(codeLensOrSelector);
				if (nativeArguments) {
					return chooseBuild(
						state,
						nativeArguments[0],
						nativeArguments[1],
						nativeArguments[2],
						nativeArguments.slice(3),
					);
				}

				return chooseBuild(
					state,
					typeof codeLensOrSelector === "string"
						? codeLensOrSelector
						: undefined,
					builds[0],
					builds[1],
					builds.slice(2),
				);
			},
		),
		...registerApplyCommands(state, [
			["versionlens.editor.onSortDependencies", "sort"],
		]),
		vscode.commands.registerCommand(
			"versionlens.editor.onShowVersionLenses",
			() => toggleVersionLenses(state, true),
		),
		vscode.commands.registerCommand(
			"versionlens.editor.onHideVersionLenses",
			() => toggleVersionLenses(state, false),
		),
		vscode.commands.registerCommand("versionlens.editor.onShowError", () =>
			showProviderError(state),
		),
		vscode.commands.registerCommand(
			"versionlens.icons.showingProgress",
			() => undefined,
		),
		vscode.commands.registerCommand(
			"versionlens.editor.onShowPrereleaseVersions",
			() => togglePrereleases(state, true),
		),
		vscode.commands.registerCommand(
			"versionlens.editor.onHidePrereleaseVersions",
			() => togglePrereleases(state, false),
		),
		...registerApplyCommands(state, [
			["versionlens.editor.onUpdateDependenciesLatest", "update"],
			["versionlens.editor.onUpdateDependenciesMajor", "updateMajor"],
			["versionlens.editor.onUpdateDependenciesMinor", "updateMinor"],
			["versionlens.editor.onUpdateDependenciesPatch", "updatePatch"],
		]),
		vscode.commands.registerCommand(
			"versionlens.editor.onAddUrlAuthentication",
			() => addAuthHeaderAndReload(state),
		),
		vscode.commands.registerCommand(
			"versionlens.editor.onRemoveUrlAuthentication",
			() => removeAuthHeaderAndReload(state),
		),
		vscode.commands.registerCommand("versionlens.editor.onCustomInstall", () =>
			runCustomInstall(state),
		),
		registerConfigurationHandler(state),
		registerCodeLensProvider(state),
	];
}

async function addAuthHeaderAndReload(state: ExtensionState) {
	if (await addAuthHeader(state)) {
		await reloadAuthBackedSession(state);
	}
}

async function removeAuthHeaderAndReload(state: ExtensionState) {
	if (await removeAuthHeader(state)) {
		await reloadAuthBackedSession(state);
	}
}

async function reloadAuthBackedSession(state: ExtensionState) {
	state.session?.clearCache();
	if (state.context?.extensionPath) {
		await recreateSession(state);
	}
	await updateContexts(state);
	refreshCodeLenses(state);
	await refreshActiveDiagnostics(state);
}
