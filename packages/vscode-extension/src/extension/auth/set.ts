import * as vscode from "vscode";
import type { ExtensionState } from "../state.ts";
import { normalizedAuthHeaderValue, normalizedRegistryUrl } from "./input.ts";
import {
	type AuthHeaderMetadata,
	authSecretKey,
	basicScheme,
	createEmptyUrlAuthenticationData,
	createUrlAuthenticationData,
	customScheme,
	getUrlAuthentication,
	notSetScheme,
	type UrlAuthenticationData,
	updateUrlAuthentication,
} from "./store.ts";

type AuthenticationScheme = "Basic" | "Custom";
type AuthHeaderStatus = "User cancelled" | "Credentials failed";

type ProviderQuickPick = vscode.QuickPickItem & {
	providerScheme: AuthenticationScheme;
};

export type AddAuthHeaderOptions = {
	authUrl?: string;
	requestUrl?: string;
};

const userCancelledStatus = "User cancelled";
const credentialsFailedStatus = "Credentials failed";
const trailingSlashes = /\/+$/;
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

export async function addAuthHeader(
	state: ExtensionState,
	options: AddAuthHeaderOptions = {},
): Promise<boolean> {
	const context = state.context;
	if (!context) {
		return false;
	}

	const retried = await retryExistingAuthHeader(state, options);
	if (retried !== undefined) {
		return retried;
	}

	const url = await promptAuthorizationUrl(options);
	if (!url) {
		await suppressAuthPrompt(context, options.authUrl);
		return false;
	}

	if (!(await confirmInsecureUrl(url))) {
		await suppressAuthPrompt(context, url);
		return false;
	}

	const provider = await vscode.window.showQuickPick(authenticationProviders, {
		placeHolder: "Choose an authentication provider",
		title: `Choose an authentication scheme for ${url}`,
	});
	if (!provider) {
		await suppressAuthPrompt(context, url);
		return false;
	}

	const value = await authorizationValue(provider.providerScheme, url);
	if (!value) {
		await suppressAuthPrompt(context, url);
		return false;
	}

	const secret = authSecretKey(context, url);
	if (!secret) {
		return false;
	}
	await context.secrets.store(secret, value);
	await writeUrlAuthentication(context, url, {
		label: provider.label,
		scheme: provider.providerScheme,
	});
	return true;
}

async function retryExistingAuthHeader(
	state: ExtensionState,
	options: AddAuthHeaderOptions,
) {
	const context = state.context;
	if (!(context && options.authUrl)) {
		return undefined;
	}
	const header = existingAuthHeader(context, options.authUrl);
	if (!header || header.scheme === notSetScheme) {
		return undefined;
	}
	const secret = authSecretKey(context, options.authUrl);
	const previousValue = secret ? await context.secrets.get(secret) : undefined;
	if (!previousValue) {
		return undefined;
	}
	const retry = await vscode.window.showWarningMessage(
		`Could not authenticate credentials with ${options.authUrl}.\n\nWould you like to re-enter your credentials?`,
		{ modal: true },
		"Yes",
	);
	if (retry !== "Yes") {
		await suppressAuthPrompt(context, options.authUrl, credentialsFailedStatus);
		return false;
	}
	const value = await authorizationValue(
		authSchemeForHeader(header, previousValue),
		options.authUrl,
	);
	if (!value) {
		await suppressAuthPrompt(context, options.authUrl);
		return false;
	}
	if (!secret) {
		return false;
	}
	await context.secrets.store(secret, value);
	await writeUrlAuthentication(
		context,
		options.authUrl,
		authHeaderMetadata(header, previousValue),
	);
	return true;
}

function authSchemeForHeader(
	header: UrlAuthenticationData,
	value: string,
): AuthenticationScheme {
	return header.scheme === basicScheme
		? basicScheme
		: authSchemeForValue(value);
}

function authSchemeForValue(value: string): AuthenticationScheme {
	return value.startsWith(`${basicScheme} `) ? basicScheme : customScheme;
}

function authHeaderMetadata(
	header: UrlAuthenticationData,
	value: string,
): AuthHeaderMetadata {
	const scheme = authSchemeForHeader(header, value);
	return {
		label: header.label ?? authHeaderLabelForScheme(scheme),
		scheme,
	};
}

function authHeaderLabelForScheme(scheme: AuthenticationScheme) {
	return scheme === basicScheme ? "Basic Auth" : "Custom Value";
}

function existingAuthHeader(
	context: NonNullable<ExtensionState["context"]>,
	url: string,
) {
	return getUrlAuthentication(context, url);
}

export function isAuthHeaderSuppressed(
	state: ExtensionState,
	options: AddAuthHeaderOptions | undefined,
) {
	const authUrl = options?.authUrl;
	const context = state.context;
	if (!(authUrl && context)) {
		return false;
	}
	return existingAuthHeader(context, authUrl)?.scheme === notSetScheme;
}

async function promptAuthorizationUrl(
	options: AddAuthHeaderOptions,
	suggestedUrl = options.authUrl,
): Promise<string | undefined> {
	const url = normalizedAuthorizationUrl(
		await vscode.window.showInputBox({
			ignoreFocusOut: true,
			placeHolder: "Authorization url",
			prompt: "Enter the authorization url for package requests",
			...(suggestedUrl === undefined ? {} : { value: suggestedUrl }),
		}),
	);
	if (!url) {
		return undefined;
	}
	const validation = await validateAuthorizationUrl(url, options.requestUrl);
	if (validation === undefined) {
		return undefined;
	}
	if (!validation) {
		return promptAuthorizationUrl(options, url);
	}
	return url;
}

function normalizedAuthorizationUrl(value: string | undefined) {
	const url = normalizedRegistryUrl(value);
	if (!url) {
		return undefined;
	}
	try {
		new URL(url);
	} catch {
		return undefined;
	}
	return url.replace(trailingSlashes, "");
}

async function validateAuthorizationUrl(
	url: string,
	requestUrl: string | undefined,
) {
	if (requestUrl === undefined) {
		return true;
	}
	const parsedRequestUrl = new URL(requestUrl);
	const parsedAuthUrl = new URL(url);
	if (parsedAuthUrl.host !== parsedRequestUrl.host) {
		return await retryOrCancel(
			"The authorization url must be in the same domain as the request url",
		);
	}
	if (!requestUrl.startsWith(url)) {
		return await retryOrCancel(
			`The authorization url must partially match the request url ${requestUrl}`,
		);
	}
	return true;
}

async function retryOrCancel(message: string) {
	return (await promptRetry(message)) ? false : undefined;
}

async function promptRetry(message: string) {
	const choice = await vscode.window.showInformationMessage(
		message,
		{ modal: true, detail: "" },
		"Retry",
	);
	return !!choice;
}

async function confirmInsecureUrl(url: string) {
	if (url.startsWith("https:")) {
		return true;
	}
	const choice = await vscode.window.showWarningMessage(
		`${url} is using the unsecure HTTP protocol.\n\nAre you sure you want to send your credentials with this url?`,
		{ modal: true },
		"Yes",
	);
	return choice === "Yes";
}

async function authorizationValue(
	scheme: AuthenticationScheme,
	url: string,
): Promise<string | undefined> {
	if (scheme === customScheme) {
		return normalizedAuthHeaderValue(
			await vscode.window.showInputBox({
				ignoreFocusOut: true,
				password: true,
				placeHolder: "Authorization value",
				prompt: `Enter the authorization value for ${url}`,
			}),
		);
	}

	return await basicAuthorizationValue(url);
}

async function basicAuthorizationValue(
	url: string,
): Promise<string | undefined> {
	const username = await vscode.window.showInputBox({
		ignoreFocusOut: true,
		password: false,
		placeHolder: "Basic auth username",
		prompt: `Enter the basic auth username for ${url}`,
	});
	if (username === undefined) {
		return undefined;
	}
	if (username.includes(":")) {
		const retry = await vscode.window.showInformationMessage(
			"You cannot have a ':' character in the user name.\n\nDo you want re-enter the username or cancel?",
			{ modal: true, detail: "" },
			"Retry",
		);
		return retry ? basicAuthorizationValue(url) : undefined;
	}

	const password = await vscode.window.showInputBox({
		ignoreFocusOut: true,
		password: true,
		placeHolder: "Basic auth password",
		prompt: `Enter the basic auth password for ${url}`,
	});
	if (password === undefined) {
		return undefined;
	}
	return Buffer.from(`${username}:${password}`).toString("base64");
}

async function suppressAuthPrompt(
	context: NonNullable<ExtensionState["context"]>,
	url: string | undefined,
	status: AuthHeaderStatus = userCancelledStatus,
) {
	if (!url) {
		return;
	}
	await updateUrlAuthentication(
		context,
		url,
		createEmptyUrlAuthenticationData(url, status),
	);
}

async function writeUrlAuthentication(
	context: NonNullable<ExtensionState["context"]>,
	url: string,
	metadataOrStatus?: AuthHeaderMetadata | AuthHeaderStatus,
) {
	const value =
		typeof metadataOrStatus === "string"
			? createEmptyUrlAuthenticationData(url, metadataOrStatus)
			: createUrlAuthenticationData(
					url,
					metadataOrStatus ?? {
						label: "Custom Value",
						scheme: customScheme,
					},
				);
	await updateUrlAuthentication(context, url, value);
}
