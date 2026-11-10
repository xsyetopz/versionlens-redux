import { dirname } from "node:path";

import { type ExtensionContext, window } from "#vscode-host";

type AuthenticationScheme = "NotSet" | "Basic" | "Custom";
type DeclinedStatus = "User cancelled" | "Credentials failed";
type UrlAuthenticationStatus = "NoStatus" | DeclinedStatus;

interface UrlAuthenticationData {
	label?: string;
	protocol: string;
	scheme: AuthenticationScheme;
	status: UrlAuthenticationStatus;
	url: string;
}

interface AuthHeaderMetadata {
	label: string;
	scheme: Exclude<AuthenticationScheme, "NotSet">;
}

const storeKey = "UrlAuthenticationStore";
const noStatus = "NoStatus";
const notSetScheme = "NotSet";
const basicScheme = "Basic";
const customScheme = "Custom";

const storeEntries = (context: ExtensionContext): [string, unknown][] => {
	const collection = context.workspaceState?.get<unknown>(storeKey, {});
	if (typeof collection === "object" && collection !== null) {
		return Object.entries(collection);
	}
	return [];
};

const urlAuthenticationEntries = (
	context: ExtensionContext | undefined,
): UrlAuthenticationData[] => {
	if (!context) {
		return [];
	}
	return storeEntries(context)
		.map((entry): unknown => entry[1])
		.filter((value): value is UrlAuthenticationData =>
			isUrlAuthenticationData(value),
		);
};

const getUrlAuthentication = (
	context: ExtensionContext | undefined,
	url: string,
): UrlAuthenticationData | undefined =>
	urlAuthenticationEntries(context).find((entry): boolean => entry.url === url);

const updateUrlAuthentication = async (
	context: ExtensionContext,
	url: string,
	value: UrlAuthenticationData,
): Promise<void> => {
	const entries = storeEntries(context).filter(([key]): boolean => key !== url);
	await context.workspaceState.update(
		storeKey,
		Object.fromEntries([...entries, [url, value]]),
	);
};

const removeUrlAuthentication = async (
	context: ExtensionContext,
	url: string,
): Promise<void> => {
	await context.workspaceState.update(
		storeKey,
		Object.fromEntries(
			storeEntries(context).filter(([key]): boolean => key !== url),
		),
	);
};

/** Stored data for a url: credentials for metadata, a decline for a status. */
const createUrlAuthenticationData = (
	url: string,
	metadata: AuthHeaderMetadata | DeclinedStatus,
): UrlAuthenticationData => {
	const { protocol } = new URL(url);
	if (typeof metadata === "string") {
		return { protocol, scheme: notSetScheme, status: metadata, url };
	}
	return {
		label: metadata.label,
		protocol,
		scheme: metadata.scheme,
		status: noStatus,
		url,
	};
};

const isNonEmpty = (value: string | undefined): value is string =>
	value !== undefined && value !== "";

const authSecretKey = (
	context: ExtensionContext,
	url: string,
): string | undefined => {
	const storagePath = context.storageUri?.path;
	const activePath = window?.activeTextEditor?.document.uri.path;
	const fromActive =
		activePath === undefined || activePath === ""
			? undefined
			: dirname(activePath);
	const folder = isNonEmpty(storagePath) ? storagePath : fromActive;
	return isNonEmpty(folder) ? `${folder}__${url}` : undefined;
};

const isUrlAuthenticationData = (
	value: unknown,
): value is UrlAuthenticationData =>
	typeof value === "object" &&
	value !== null &&
	"url" in value &&
	typeof value.url === "string" &&
	"scheme" in value &&
	typeof value.scheme === "string" &&
	"status" in value &&
	typeof value.status === "string" &&
	"protocol" in value &&
	typeof value.protocol === "string";

export type {
	AuthenticationScheme,
	AuthHeaderMetadata,
	DeclinedStatus,
	UrlAuthenticationData,
};
export {
	authSecretKey,
	basicScheme,
	createUrlAuthenticationData,
	customScheme,
	getUrlAuthentication,
	noStatus,
	notSetScheme,
	removeUrlAuthentication,
	updateUrlAuthentication,
	urlAuthenticationEntries,
};
