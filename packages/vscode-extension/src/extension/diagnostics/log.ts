import type { ExtensionState } from "../state.ts";

function logProviderError(state: ExtensionState, error: unknown): void {
  console.error(error);
  state.ui.outputChannel?.appendLine(errorText(error));
}

function errorText(error: unknown): string {
  if (error instanceof Error) {
    return error.stack ?? error.message;
  }
  return String(error);
}

export { logProviderError };
