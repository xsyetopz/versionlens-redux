import { DidChangeConfigurationNotification } from "vscode-languageclient/node";
import { applyDiagnosticsVisibility, refreshCodeLenses } from "../client.ts";
import type { ExtensionState } from "../state.ts";
import { updateContexts } from "./contexts.ts";

export async function toggleVersionLenses(
  state: ExtensionState,
  next: boolean,
): Promise<void> {
  state.flags.showVersionLenses = next;
  state.flags.diagnosticsHidden = !next;
  await updateContexts(state);
  refreshCodeLenses(state);
  applyDiagnosticsVisibility(state);
}

/**
 * The server reads `showPrereleasesOnStartup` from the settings it pulls
 * (see `client.ts`), so the toggle asks it to pull them again.
 */
export async function togglePrereleases(
  state: ExtensionState,
  next: boolean,
): Promise<void> {
  state.flags.showPrereleases = next;
  await notifyConfigurationChanged(state);
  await updateContexts(state);
}

export async function notifyConfigurationChanged(
  state: ExtensionState,
): Promise<void> {
  if (state.client?.isRunning()) {
    await state.client.sendNotification(
      DidChangeConfigurationNotification.type,
      { settings: null },
    );
  }
}
