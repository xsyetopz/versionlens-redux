import * as vscode from "vscode";
import { analyzeDocument } from "../diagnostics.ts";
import { activeFileDocument } from "../documents.ts";
import type { ExtensionState } from "../state.ts";
import { customInstallTaskLabel } from "../tasks.ts";

export async function updateContexts(state: ExtensionState) {
	const activeDocument = activeFileDocument();
	const activeOutput = activeDocument
		? analyzeDocument(state, activeDocument)
		: undefined;
	const providerActive = activeOutput?.activeProviderName ?? false;

	await vscode.commands.executeCommand(
		"setContext",
		"versionlens.show",
		state.flags.showVersionLenses,
	);
	await vscode.commands.executeCommand(
		"setContext",
		"versionlens.showPrereleases",
		state.flags.showPrereleases,
	);
	await vscode.commands.executeCommand(
		"setContext",
		"versionlens.showOutdated",
		state.flags.showOutdated,
	);
	await vscode.commands.executeCommand(
		"setContext",
		"versionlens.providerActive",
		providerActive,
	);
	await vscode.commands.executeCommand(
		"setContext",
		"versionlens.providerBusy",
		state.flags.providerBusy,
	);
	await vscode.commands.executeCommand(
		"setContext",
		"versionlens.providerError",
		state.flags.providerError,
	);
	await vscode.commands.executeCommand(
		"setContext",
		"versionlens.showCustomInstall",
		providerActive && Boolean(customInstallTaskLabel(state, activeDocument)),
	);
	await vscode.commands.executeCommand(
		"setContext",
		"versionlens.showSortAlphabetically",
		providerActive && (activeOutput?.canSortDependencies ?? false),
	);

	return providerActive;
}
