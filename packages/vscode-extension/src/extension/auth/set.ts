import { type QuickPickItem, window } from "#vscode-host";

import { optionalProperty } from "../config/optional.ts";
import type { ExtensionState } from "../state.ts";
import {
	normalizedAuthorizationUrl,
	validateAuthorizationUrl,
} from "./authorization-url.ts";
import {
	type AuthHeaderMetadata,
	authSecretKey,
	basicScheme,
	createUrlAuthenticationData,
	customScheme,
	type DeclinedStatus,
	getUrlAuthentication,
	notSetScheme,
	updateUrlAuthentication,
} from "./store.ts";
import { authorizationValue, confirmInsecureUrl } from "./value.ts";

type AuthContext = NonNullable<ExtensionState["context"]>;

type ProviderQuickPick = QuickPickItem & {
	providerScheme: AuthHeaderMetadata["scheme"];
};

interface AddAuthHeaderOptions {
	authUrl?: string;
	requestUrl?: string;
}

const authenticationProviders: ProviderQuickPick[] = [
	{
		detail: "Authenticate using basic auth credentials",
		label: "Basic Auth",
		providerScheme: basicScheme,
	},
	{
		detail: "Authenticate using a custom authorization value",
		label: "Custom Value",
		providerScheme: customScheme,
	},
];

const addAuthHeader = async (
	state: ExtensionState,
	options: AddAuthHeaderOptions = {},
): Promise<boolean> => {
	const { context } = state;
	if (!context) {
		return false;
	}

	const retried = await retryExistingAuthHeader(context, options);
	if (retried !== null) {
		return retried;
	}

	const url = await promptAuthorizationUrl(options);
	if (url === null) {
		await suppressAuthPrompt(context, options.authUrl);
		return false;
	}

	if (!(await confirmInsecureUrl(url))) {
		await suppressAuthPrompt(context, url);
		return false;
	}

	const provider = await window.showQuickPick(authenticationProviders, {
		placeHolder: "Choose an authentication provider",
		title: `Choose an authentication scheme for ${url}`,
	});
	if (!provider) {
		await suppressAuthPrompt(context, url);
		return false;
	}

	return storeCredentials(context, url, {
		label: provider.label,
		scheme: provider.providerScheme,
	});
};

/** Asks for the value and stores it with the secret; false when cancelled. */
const storeCredentials = async (
	context: AuthContext,
	url: string,
	metadata: AuthHeaderMetadata,
	presetSecret?: string,
): Promise<boolean> => {
	const value = await authorizationValue(metadata.scheme, url);
	if (value.kind === "cancelled") {
		await suppressAuthPrompt(context, url);
		return false;
	}
	const secret = presetSecret ?? authSecretKey(context, url);
	if (secret === undefined || secret === "") {
		return false;
	}
	await context.secrets.store(secret, value.value);
	await updateUrlAuthentication(
		context,
		url,
		createUrlAuthenticationData(url, {
			label: metadata.label,
			scheme: metadata.scheme,
		}),
	);
	return true;
};

const retryExistingAuthHeader = async (
	context: AuthContext,
	{ authUrl }: AddAuthHeaderOptions,
): Promise<boolean | null> => {
	if (authUrl === undefined || authUrl === "") {
		return null;
	}
	const header = getUrlAuthentication(context, authUrl);
	if (header === undefined || header.scheme === notSetScheme) {
		return null;
	}
	const secret = authSecretKey(context, authUrl);
	const previousValue =
		secret === undefined || secret === ""
			? undefined
			: await context.secrets.get(secret);
	if (previousValue === undefined || previousValue === "") {
		return null;
	}
	const retry = await window.showWarningMessage(
		`Could not authenticate credentials with ${authUrl}.\n\nWould you like to re-enter your credentials?`,
		{ modal: true },
		"Yes",
	);
	if (retry !== "Yes") {
		await suppressAuthPrompt(context, authUrl, "Credentials failed");
		return false;
	}
	if (header.scheme === basicScheme || previousValue.startsWith("Basic ")) {
		return storeCredentials(
			context,
			authUrl,
			{
				label: header.label ?? "Basic Auth",
				scheme: basicScheme,
			},
			secret,
		);
	}
	return storeCredentials(
		context,
		authUrl,
		{
			label: header.label ?? "Custom Value",
			scheme: customScheme,
		},
		secret,
	);
};

const promptAuthorizationUrl = async (
	options: AddAuthHeaderOptions,
	suggestedUrl = options.authUrl,
): Promise<string | null> => {
	const url = normalizedAuthorizationUrl(
		await window.showInputBox({
			ignoreFocusOut: true,
			placeHolder: "Authorization url",
			prompt: "Enter the authorization url for package requests",
			...optionalProperty("value", suggestedUrl),
		}),
	);
	if (url === undefined) {
		return null;
	}
	const validation = await validateAuthorizationUrl(url, options.requestUrl);
	if (validation === undefined) {
		return null;
	}
	if (!validation) {
		return promptAuthorizationUrl(options, url);
	}
	return url;
};

const suppressAuthPrompt = async (
	context: AuthContext,
	url: string | undefined,
	status: DeclinedStatus = "User cancelled",
): Promise<void> => {
	if (url !== undefined && url !== "") {
		await updateUrlAuthentication(
			context,
			url,
			createUrlAuthenticationData(url, status),
		);
	}
};

export type { AddAuthHeaderOptions };
export { addAuthHeader };
