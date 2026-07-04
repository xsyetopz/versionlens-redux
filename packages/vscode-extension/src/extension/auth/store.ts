import { dirname } from "node:path";
import * as vscode from "vscode";

type AuthenticationScheme = "NotSet" | "Basic" | "Custom";
type UrlAuthenticationStatus =
	| "NoStatus"
	| "User cancelled"
	| "Credentials failed";

export type UrlAuthenticationData = {
	label?: string;
	protocol: string;
	scheme: AuthenticationScheme;
	status: UrlAuthenticationStatus;
	url: string;
};

export type AuthHeaderMetadata = {
	label: string;
	scheme: Exclude<AuthenticationScheme, "NotSet">;
};

export const urlAuthenticationStoreKey = "UrlAuthenticationStore";
export const authorizationHeaderName = "Authorization";
export const noStatus = "NoStatus";
export const notSetScheme = "NotSet";
export const basicScheme = "Basic";
export const customScheme = "Custom";

export function urlAuthenticationEntries(
	context: vscode.ExtensionContext | undefined,
): UrlAuthenticationData[] {
	if (!context) {
		return [];
	}
	return collectionValues(context).filter(isUrlAuthenticationData);
}

export function getUrlAuthentication(
	context: vscode.ExtensionContext | undefined,
	url: string,
): UrlAuthenticationData | undefined {
	if (!context) {
		return undefined;
	}
	const entry = collectionEntries(context).find(([key]) => key === url)?.[1];
	return isUrlAuthenticationData(entry) ? entry : undefined;
}

export async function updateUrlAuthentication(
	context: vscode.ExtensionContext,
	url: string,
	value: UrlAuthenticationData,
) {
	const entries = collectionEntries(context).filter(([key]) => key !== url);
	await context.workspaceState.update(
		urlAuthenticationStoreKey,
		Object.fromEntries([...entries, [url, value]]),
	);
}

export async function removeUrlAuthentication(
	context: vscode.ExtensionContext,
	url: string,
) {
	await context.workspaceState.update(
		urlAuthenticationStoreKey,
		Object.fromEntries(
			collectionEntries(context).filter(([key]) => key !== url),
		),
	);
}

export function createUrlAuthenticationData(
	url: string,
	metadata: AuthHeaderMetadata,
): UrlAuthenticationData {
	return {
		label: metadata.label,
		protocol: new URL(url).protocol,
		scheme: metadata.scheme,
		status: noStatus,
		url,
	};
}

export function createEmptyUrlAuthenticationData(
	url: string,
	status: Exclude<UrlAuthenticationStatus, "NoStatus"> = "User cancelled",
): UrlAuthenticationData {
	return {
		protocol: new URL(url).protocol,
		scheme: notSetScheme,
		status,
		url,
	};
}

export function authSecretKey(
	context: vscode.ExtensionContext,
	url: string,
): string | undefined {
	const resourceFolderPath = resourceFolderPathForAuth(context);
	return resourceFolderPath ? `${resourceFolderPath}__${url}` : undefined;
}

export function resourceFolderPathForAuth(
	context: vscode.ExtensionContext,
): string | undefined {
	if (context.storageUri?.path) {
		return context.storageUri.path;
	}

	const activePath = vscode.window?.activeTextEditor?.document.uri.path;
	return activePath ? dirname(activePath) : undefined;
}

function collectionEntries(
	context: vscode.ExtensionContext,
): [string, unknown][] {
	const workspaceState = context.workspaceState;
	if (!workspaceState) {
		return [];
	}
	const collection = workspaceState.get<unknown>(urlAuthenticationStoreKey, {});
	return typeof collection === "object" && collection !== null
		? Object.entries(collection)
		: [];
}

function collectionValues(context: vscode.ExtensionContext): unknown[] {
	return collectionEntries(context).map((entry) => entry[1]);
}

function isUrlAuthenticationData(
	value: unknown,
): value is UrlAuthenticationData {
	if (!(typeof value === "object" && value !== null)) {
		return false;
	}
	const entry = value as UrlAuthenticationData;
	return (
		typeof entry.url === "string" &&
		typeof entry.scheme === "string" &&
		typeof entry.status === "string" &&
		typeof entry.protocol === "string"
	);
}
