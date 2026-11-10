import { window } from "#vscode-host";

const trailingSlashes = /\/+$/u;
const registryUrlPrefix = /^https?:\/\//u;

const normalizedRequiredInput = (
	value: string | undefined,
): string | undefined => {
	const trimmed = value?.trim();
	return trimmed === "" ? undefined : trimmed;
};

const normalizedAuthorizationUrl = (
	value: string | undefined,
): string | undefined => {
	const url = normalizedRequiredInput(value);
	return url !== undefined && registryUrlPrefix.test(url) && URL.canParse(url)
		? url.replace(trailingSlashes, "")
		: undefined;
};

const validateAuthorizationUrl = (
	url: string,
	requestUrl: string | undefined,
): Promise<boolean | undefined> => {
	if (requestUrl === undefined) {
		return Promise.resolve(true);
	}
	const parsedRequestUrl = new URL(requestUrl);
	const parsedAuthUrl = new URL(url);
	if (!sameOrigin(parsedAuthUrl, parsedRequestUrl)) {
		return retryOrCancel(
			"The authorization url must have the same scheme, domain, and port as the request url",
		);
	}
	if (!pathContains(parsedAuthUrl.pathname, parsedRequestUrl.pathname)) {
		return retryOrCancel(
			`The authorization url must partially match the request url ${requestUrl}`,
		);
	}
	return Promise.resolve(true);
};

const sameOrigin = (authUrl: URL, requestUrl: URL): boolean =>
	authUrl.protocol === requestUrl.protocol &&
	authUrl.hostname === requestUrl.hostname &&
	effectivePort(authUrl) === effectivePort(requestUrl);

const effectivePort = (url: URL): string => {
	if (url.port) {
		return url.port;
	}
	if (url.protocol === "http:") {
		return "80";
	}
	if (url.protocol === "https:") {
		return "443";
	}
	return "";
};

const pathContains = (authPath: string, requestPath: string): boolean => {
	const normalizedAuthPath = authPath.replace(trailingSlashes, "");
	const normalizedRequestPath = requestPath.replace(trailingSlashes, "");
	return (
		normalizedAuthPath === "" ||
		normalizedRequestPath === normalizedAuthPath ||
		normalizedRequestPath.startsWith(`${normalizedAuthPath}/`)
	);
};

const retryOrCancel = async (message: string): Promise<false | undefined> => {
	const choice = await window.showInformationMessage(
		message,
		{ modal: true, detail: "" },
		"Retry",
	);
	return choice === undefined ? undefined : false;
};

export {
	normalizedAuthorizationUrl,
	normalizedRequiredInput,
	validateAuthorizationUrl,
};
