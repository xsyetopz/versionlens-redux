import * as vscode from "vscode";
import { configuredValue } from "./configured.ts";

export function cacheDurationMinutes() {
	const config = vscode.workspace.getConfiguration("versionlens");
	const minutes = configuredValue<number | null | undefined>(
		"caching.duration",
		config,
	);
	return minutes === undefined ? undefined : (minutes ?? 0);
}
