import type { TextDocument } from "#vscode-host";
import type { ExtensionState } from "../state.ts";
import { analyzeDocument } from "./analyze.ts";

export function dependencySnapshot(
  state: ExtensionState,
  document: TextDocument,
): string {
  return analyzeDocument(state, document)?.dependencySignature ?? "";
}

export function rememberDependencySnapshot(
  state: ExtensionState,
  document: TextDocument,
  snapshot: string,
): void {
  const key = document.uri.toString();

  if (document.isDirty) {
    state.snapshots.editedDependencies.set(key, snapshot);
    const saved = state.snapshots.savedDependencies.get(key);
    state.flags.showOutdated = saved !== snapshot;
    if (saved === undefined) {
      state.flags.showOutdated = snapshot !== "";
    }
    return;
  }

  state.snapshots.savedDependencies.set(key, snapshot);
  state.snapshots.editedDependencies.delete(key);
  state.flags.showOutdated = false;
}
