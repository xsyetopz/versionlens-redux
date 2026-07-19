import { type TextDocument, window } from "#vscode-host";
import type { ExtensionState } from "../state.ts";
import { analyzeDocument } from "./analyze.ts";
import { toDiagnostic } from "./convert.ts";
import { resolveDocumentForDiagnostics } from "./resolve.ts";
import { rememberDependencySnapshot } from "./snapshot.ts";

export async function refreshActiveDiagnostics(
  state: ExtensionState,
): Promise<void> {
  const document = window.activeTextEditor?.document;
  if (document) {
    await refreshDiagnostics(state, document);
  }
}

export async function refreshDiagnostics(
  state: ExtensionState,
  document: TextDocument,
): Promise<void> {
  if (!(state.ui.diagnostics && state.flags.showVersionLenses)) {
    return;
  }

  if (!(await resolveDocumentForDiagnostics(state, document))) {
    return;
  }
  const output = analyzeDocument(state, document);
  if (!output) {
    return;
  }
  rememberDependencySnapshot(state, document, output.dependencySignature);
  state.ui.diagnostics.set(document.uri, output.diagnostics.map(toDiagnostic));
}
