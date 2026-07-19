import { languages, window } from "#vscode-host";
import type { ExtensionState } from "../state.ts";

export function initializeUi(state: ExtensionState): void {
  state.ui.diagnostics = languages.createDiagnosticCollection(
    "versionlens-vulnerabilities",
  );
  state.ui.outputChannel = window.createOutputChannel("Version Lens");
}

export function disposeUi(state: ExtensionState): void {
  state.ui.diagnostics?.clear();
  state.ui.diagnostics = undefined;
  state.ui.outputChannel?.dispose();
  state.ui.outputChannel = undefined;
  state.ui.codeLensRefresh?.dispose();
  state.ui.codeLensRefresh = undefined;
  state.ui.codeLensProvider = undefined;
  state.ui.resetCodeLensResolutions = undefined;
}
