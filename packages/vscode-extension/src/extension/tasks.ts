import type { TextDocument } from "#vscode-host";
import { fileDocument } from "./documents.ts";
import { manifestInfo } from "./manifest.ts";
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

  const info = await manifestInfo(state, file);
  if (!info?.supported) {
    return;
  }

  await runInstallTaskIfDependenciesChanged(state, file);
}
