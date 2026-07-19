import { commands } from "#vscode-host";
import type { ExtensionState } from "../state.ts";

function increaseProviderBusy(state: ExtensionState): void {
  state.flags.providerBusy += 1;
  publishProviderState(state);
}

function decreaseProviderBusy(state: ExtensionState): void {
  state.flags.providerBusy = Math.max(0, state.flags.providerBusy - 1);
  publishProviderState(state);
}

function clearProviderBusy(state: ExtensionState): void {
  state.flags.providerBusy = 0;
  publishProviderState(state);
}

function setProviderError(state: ExtensionState): void {
  state.flags.providerError = true;
  publishProviderState(state);
}

function clearProviderError(state: ExtensionState): void {
  state.flags.providerError = false;
  publishProviderState(state);
}

function setProviderState(
  state: ExtensionState,
  busy: boolean,
  error: boolean,
): void {
  if (busy) {
    state.flags.providerBusy += 1;
  } else {
    state.flags.providerBusy = Math.max(0, state.flags.providerBusy - 1);
  }
  state.flags.providerError = error;
  publishProviderState(state);
}

function publishProviderState(state: ExtensionState): void {
  commands.executeCommand(
    "setContext",
    "versionlens.providerBusy",
    state.flags.providerBusy,
  );
  commands.executeCommand(
    "setContext",
    "versionlens.providerError",
    state.flags.providerError,
  );
}

export {
  clearProviderBusy,
  clearProviderError,
  decreaseProviderBusy,
  increaseProviderBusy,
  setProviderError,
  setProviderState,
};
