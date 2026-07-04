import type { NativeHttpHeader } from "../native/config.ts";
import type { ExtensionState } from "../state.ts";
import {
	authorizationHeaderName,
	authSecretKey,
	basicScheme,
	customScheme,
	noStatus,
	type UrlAuthenticationData,
	urlAuthenticationEntries,
} from "./store.ts";

type ResolvedAuthHeader = {
	name: string;
	url?: string;
	value: string;
};

export async function authHeaders(
	state: ExtensionState,
): Promise<NativeHttpHeader[]> {
	const context = state.context;
	if (!context) {
		return [];
	}

	const headers = await Promise.all(
		urlAuthenticationEntries(context).map((entry) =>
			urlAuthenticationHeader(context, entry),
		),
	);
	return headers
		.filter((header): header is ResolvedAuthHeader => header !== undefined)
		.map((header) => ({
			name: header.name,
			...(header.url === undefined ? {} : { url: header.url }),
			value: header.value,
		}));
}

async function urlAuthenticationHeader(
	context: NonNullable<ExtensionState["context"]>,
	entry: UrlAuthenticationData,
): Promise<ResolvedAuthHeader | undefined> {
	if (
		entry.status !== noStatus ||
		!(entry.scheme === basicScheme || entry.scheme === customScheme)
	) {
		return undefined;
	}

	const secretKey = authSecretKey(context, entry.url);
	const token = secretKey ? await context.secrets.get(secretKey) : undefined;
	if (!token) {
		return undefined;
	}

	return {
		name: authorizationHeaderName,
		url: entry.url,
		value: entry.scheme === basicScheme ? `${basicScheme} ${token}` : token,
	};
}
