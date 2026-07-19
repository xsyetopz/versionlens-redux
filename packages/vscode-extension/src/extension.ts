import type { ExtensionContext } from "#vscode-host";
import { activateExtension } from "./extension/lifecycle/activate.ts";
import { deactivateExtension } from "./extension/lifecycle/deactivate.ts";
import { createExtensionState } from "./extension/state.ts";

const state = createExtensionState();

export async function activate(context: ExtensionContext): Promise<void> {
  await activateExtension(state, context);
}

export function deactivate(): void {
  deactivateExtension(state);
}
