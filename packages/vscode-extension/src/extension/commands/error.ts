import { window } from "#vscode-host";
import {
  clearProviderBusy,
  clearProviderError,
} from "../diagnostics/provider.ts";
import type { ExtensionState } from "../state.ts";

export function showProviderError(state: ExtensionState): void {
  state.ui.outputChannel?.show();
  clearProviderError(state);
  clearProviderBusy(state);
  const document = window.activeTextEditor?.document;
  if (document) {
    window.showTextDocument(document);
  }
}
