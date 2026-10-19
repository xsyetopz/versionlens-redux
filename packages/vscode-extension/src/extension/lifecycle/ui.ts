import { window } from "#vscode-host";
import type { ExtensionState } from "../state.ts";

export function initializeUi(state: ExtensionState): void {
  state.ui.outputChannel = window.createOutputChannel("Version Lens", {
    log: true,
  });
}

export function disposeUi(state: ExtensionState): void {
  state.ui.outputChannel?.dispose();
  state.ui.outputChannel = undefined;
}
