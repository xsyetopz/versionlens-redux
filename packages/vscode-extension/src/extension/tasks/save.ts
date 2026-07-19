import { type TextDocument, workspace } from "#vscode-host";
import { dependencySnapshot } from "../diagnostics/snapshot.ts";
import type { ExtensionState } from "../state.ts";
import { customInstallTaskLabel } from "./custom-install.ts";
import { runTask } from "./runner.ts";

async function runInstallTaskIfDependenciesChanged(
  state: ExtensionState,
  document: TextDocument,
): Promise<boolean> {
  const key = document.uri.toString();
  const previous = state.snapshots.savedDependencies.get(key);
  const current =
    state.snapshots.editedDependencies.get(key) ??
    dependencySnapshot(state, document);

  if (previous === undefined || previous === current) {
    rememberSavedDependencies(state, key, current);
    return true;
  }

  const label = customInstallTaskLabel(state, document);
  if (!label) {
    rememberSavedDependencies(state, key, current);
    return true;
  }

  state.flags.showOutdated = false;
  const result = await runTask(
    label,
    workspace.getWorkspaceFolder(document.uri),
  );
  if (result.kind === "completed" && result.exitCode === 0) {
    rememberSavedDependencies(state, key, current);
    return true;
  }

  state.flags.showOutdated = true;
  return false;
}

function rememberSavedDependencies(
  state: ExtensionState,
  key: string,
  current: string,
): void {
  state.snapshots.savedDependencies.set(key, current);
  state.snapshots.editedDependencies.delete(key);
  state.flags.showOutdated = false;
}

export { runInstallTaskIfDependenciesChanged };
