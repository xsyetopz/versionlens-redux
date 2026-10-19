import { stopClient } from "../client.ts";
import type { ExtensionState } from "../state.ts";
import { disposeUi } from "./ui.ts";

export async function deactivateExtension(
  state: ExtensionState,
): Promise<void> {
  await stopClient(state);
  disposeUi(state);
  for (const subscription of state.context?.subscriptions ?? []) {
    subscription.dispose();
  }
  state.context?.subscriptions.splice(0);
  state.snapshots.savedDependencies.clear();
  state.snapshots.editedDependencies.clear();
  state.context = undefined;
}
