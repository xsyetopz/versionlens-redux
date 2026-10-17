import type { ExtensionContext } from "#vscode-host";
import { createLanguageClient } from "./extension/client.ts";
import { activateExtension } from "./extension/lifecycle/activate.ts";
import { deactivateExtension } from "./extension/lifecycle/deactivate.ts";
import { createExtensionState } from "./extension/state.ts";

// Exported ahead of the LSP client wiring (task 6.2); activation does not call it yet.
export { createLanguageClient };

const state = createExtensionState();

export async function activate(context: ExtensionContext): Promise<void> {
  await activateExtension(state, context);
}

export function deactivate(): void {
  deactivateExtension(state);
}
