import * as vscode from "vscode";
import type { ExtensionState } from "../state.ts";

export function increaseProviderBusy(state: ExtensionState) {
	state.flags.providerBusy += 1;
	publishProviderState(state);
}

export function decreaseProviderBusy(state: ExtensionState) {
	state.flags.providerBusy = Math.max(0, state.flags.providerBusy - 1);
	publishProviderState(state);
}

export function clearProviderBusy(state: ExtensionState) {
	state.flags.providerBusy = 0;
	publishProviderState(state);
}

export function setProviderError(state: ExtensionState) {
	state.flags.providerError = true;
	publishProviderState(state);
}

export function clearProviderError(state: ExtensionState) {
	state.flags.providerError = false;
	publishProviderState(state);
}

export function setProviderState(
	state: ExtensionState,
	busy: boolean,
	error: boolean,
) {
	state.flags.providerBusy = busy
		? state.flags.providerBusy + 1
		: Math.max(0, state.flags.providerBusy - 1);
	state.flags.providerError = error;
	publishProviderState(state);
}

function publishProviderState(state: ExtensionState) {
	vscode.commands.executeCommand(
		"setContext",
		"versionlens.providerBusy",
		state.flags.providerBusy,
	);
	vscode.commands.executeCommand(
		"setContext",
		"versionlens.providerError",
		state.flags.providerError,
	);
}
