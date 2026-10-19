import { commands } from "#vscode-host";
import type { ExtensionState } from "../state.ts";

interface StatusParams {
  status: "busy" | "error" | "idle";
  uri: string;
}

/** Applies a `versionlens/status` notification to the busy and error keys. */
function applyStatus(
  state: ExtensionState,
  { status, uri }: StatusParams,
): Promise<void> {
  if (status === "idle") {
    state.activity.delete(uri);
  } else {
    state.activity.set(uri, status);
  }
  return publishProviderState(state);
}

function clearProviderError(state: ExtensionState): Promise<void> {
  for (const [uri, activity] of state.activity) {
    if (activity === "error") {
      state.activity.delete(uri);
    }
  }
  return publishProviderState(state);
}

function resetProviderState(state: ExtensionState): Promise<void> {
  state.activity.clear();
  return publishProviderState(state);
}

async function publishProviderState(state: ExtensionState): Promise<void> {
  const activities = [...state.activity.values()];
  state.flags.providerBusy = activities.filter(
    (activity): boolean => activity === "busy",
  ).length;
  state.flags.providerError = activities.includes("error");
  await commands.executeCommand(
    "setContext",
    "versionlens.providerBusy",
    state.flags.providerBusy,
  );
  await commands.executeCommand(
    "setContext",
    "versionlens.providerError",
    state.flags.providerError,
  );
}

export type { StatusParams };
export { applyStatus, clearProviderError, resetProviderState };
