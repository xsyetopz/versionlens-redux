import { window } from "#vscode-host";
import { clearProviderError } from "../session/status.ts";
import type { ExtensionState } from "../state.ts";

export async function showProviderError(state: ExtensionState): Promise<void> {
  state.ui.outputChannel?.show();
  await clearProviderError(state);
  const document = window.activeTextEditor?.document;
  if (document) {
    await window.showTextDocument(document);
  }
}

/** Logs a failure and tells the user about it. */
export async function reportError(
  state: ExtensionState,
  title: string,
  error: unknown,
): Promise<void> {
  let detail = String(error);
  if (error instanceof Error) {
    detail = error.message;
  }
  state.ui.outputChannel?.error(`${title}: ${detail}`);
  // Not awaited: the promise settles only when the user closes the message.
  void window.showErrorMessage(`VersionLens: ${title}: ${detail}`);
}
