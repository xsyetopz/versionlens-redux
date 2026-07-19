import type { Disposable } from "#vscode-host";
import { refreshActiveDiagnostics } from "../diagnostics/refresh.ts";
import type { ExtensionState } from "../state.ts";

const REFRESH_INTERVAL_MS = 900_000;

type UnrefTimer = ReturnType<typeof setInterval> & { unref?: () => void };

export function registerRefreshTimer(
  state: ExtensionState,
  intervalMs = REFRESH_INTERVAL_MS,
): Disposable {
  const timer: UnrefTimer = setInterval(
    (): Promise<void> => refreshActiveDiagnostics(state),
    intervalMs,
  );
  timer.unref?.();
  return { dispose: (): void => clearInterval(timer) };
}
