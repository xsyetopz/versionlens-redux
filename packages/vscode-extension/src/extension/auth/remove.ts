import * as vscode from "vscode";
import type { ExtensionState } from "../state.ts";
import {
	authSecretKey,
	noStatus,
	notSetScheme,
	removeUrlAuthentication,
	type UrlAuthenticationData,
	urlAuthenticationEntries,
} from "./store.ts";

type AuthHeaderQuickPick = vscode.QuickPickItem & {
	entry: UrlAuthenticationData;
};

export async function removeAuthHeader(
	state: ExtensionState,
): Promise<boolean> {
	const context = state.context;
	const choices: AuthHeaderQuickPick[] = urlAuthenticationChoices(context);
	const picked = await vscode.window.showQuickPick<AuthHeaderQuickPick>(
		choices,
		{
			canPickMany: true,
			placeHolder: "Choose which urls to remove",
			title: "Clear url authentication",
		},
	);
	const pickedItems = picked ?? [];
	if (!(context && pickedItems.length > 0)) {
		return false;
	}

	for (const item of pickedItems) {
		await removeUrlAuthentication(context, item.entry.url);
		if (item.entry.scheme !== notSetScheme) {
			const secret = authSecretKey(context, item.entry.url);
			if (secret) {
				await context.secrets.delete(secret);
			}
		}
	}
	return true;
}

function urlAuthenticationChoices(
	context: vscode.ExtensionContext | undefined,
): AuthHeaderQuickPick[] {
	return urlAuthenticationEntries(context).map((entry) => ({
		entry,
		label: entry.url,
		...urlAuthenticationDetail(entry),
	}));
}

function urlAuthenticationDetail(entry: UrlAuthenticationData) {
	const detail: string[] = [];
	if (entry.scheme !== notSetScheme) {
		detail.push(entry.protocol === "http:" ? "Unsecured" : "Secure");
		if (entry.label) {
			detail.push(entry.label);
		}
	}
	if (entry.status !== noStatus) {
		detail.push(`(${entry.status})`);
	}
	return detail.length > 0 ? { detail: detail.join(" ") } : {};
}
