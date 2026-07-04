import type * as vscode from "vscode";
import { refreshActiveDiagnostics } from "../diagnostics.ts";
import type { ExtensionState } from "../state.ts";

const REFRESH_INTERVAL_MS = 15 * 60 * 1000;

type UnrefTimer = ReturnType<typeof setInterval> & { unref?: () => void };

export function registerRefreshTimer(
	state: ExtensionState,
	intervalMs = REFRESH_INTERVAL_MS,
): vscode.Disposable {
	const timer: UnrefTimer = setInterval(
		() => refreshActiveDiagnostics(state),
		intervalMs,
	);
	timer.unref?.();
	return { dispose: () => clearInterval(timer) };
}
