import type * as vscode from "vscode";
import { activateExtension } from "./extension/lifecycle/activate.ts";
import { deactivateExtension } from "./extension/lifecycle/deactivate.ts";
import { createExtensionState } from "./extension/state.ts";

const state = createExtensionState();

export async function activate(context: vscode.ExtensionContext) {
	await activateExtension(state, context);
}

export function deactivate() {
	deactivateExtension(state);
}
