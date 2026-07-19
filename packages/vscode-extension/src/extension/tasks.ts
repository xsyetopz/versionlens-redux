import type { TextDocument } from "#vscode-host";
import { analyzeDocument } from "./diagnostics/analyze.ts";
import { refreshDiagnostics } from "./diagnostics/refresh.ts";
import { fileDocument } from "./documents/file.ts";
import type { ExtensionState } from "./state.ts";
import { runInstallTaskIfDependenciesChanged } from "./tasks/save.ts";

export async function handleDidSaveTextDocument(
  state: ExtensionState,
  document: TextDocument,
): Promise<void> {
  const file = fileDocument(document);
  if (!file) {
    return;
  }

  const output = analyzeDocument(state, file);
  if (!output?.isSupportedManifest) {
    return;
  }

  const saved = await runInstallTaskIfDependenciesChanged(state, file);
  if (saved) {
    await refreshDiagnostics(state, file);
  }
}
