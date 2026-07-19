import { refreshActiveDiagnostics } from "../diagnostics/refresh.ts";
import { recreateSessions } from "../session/registry.ts";
import type { ExtensionState } from "../state.ts";
import { refreshCodeLenses } from "./codelens.ts";
import { updateContexts } from "./contexts.ts";

export async function toggleVersionLenses(
  state: ExtensionState,
  next: boolean,
): Promise<void> {
  state.flags.showVersionLenses = next;
  await updateContexts(state);
  refreshCodeLenses(state);
  if (next) {
    await refreshActiveDiagnostics(state);
  } else {
    state.ui.diagnostics?.clear();
  }
}

export async function togglePrereleases(
  state: ExtensionState,
  next: boolean,
): Promise<void> {
  state.flags.showPrereleases = next;
  if (!(await recreateSessions(state))) {
    return;
  }
  await updateContexts(state);
  refreshCodeLenses(state);
}
