import type { ExtensionContext } from "#vscode-host";

import { activateExtension } from "./extension/lifecycle/activate.ts";
import { deactivateExtension } from "./extension/lifecycle/deactivate.ts";
import { createExtensionState } from "./extension/state.ts";

const state = createExtensionState();

/** Starts the extension: the server client, commands and listeners. */
export const activate = async (context: ExtensionContext): Promise<void> => {
	await activateExtension(state, context);
};

/** Stops the language server when the extension is deactivated. */
export const deactivate = (): Promise<void> => deactivateExtension(state);
