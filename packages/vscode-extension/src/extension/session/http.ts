import * as vscode from "vscode";
import { authHeaders } from "../auth.ts";
import type { ExtensionState } from "../state.ts";
import { configuredValue } from "./configured.ts";

export async function httpConfig(state: ExtensionState) {
	const http = vscode.workspace.getConfiguration("http");
	const versionlens = vscode.workspace.getConfiguration("versionlens");
	const proxy = configuredValue<string | undefined>("proxy", http);
	const strictSsl = configuredValue<boolean | null | undefined>(
		"http.strictSSL",
		versionlens,
	);
	const headers = await authHeaders(state);

	return {
		...(headers.length === 0 ? {} : { authHeaders: headers }),
		...(proxy === undefined ? {} : { proxy }),
		...(strictSsl === undefined || strictSsl === null ? {} : { strictSsl }),
	};
}
