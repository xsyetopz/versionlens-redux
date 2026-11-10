import type { ExtensionState } from "../state.ts";
import { addAuthHeader } from "./set.ts";
import {
	authSecretKey,
	basicScheme,
	noStatus,
	notSetScheme,
	type UrlAuthenticationData,
	urlAuthenticationEntries,
} from "./store.ts";

/** Params of the server's `versionlens/authorization` request. */
interface AuthorizationParams {
	host: string;
	status: number;
}

/** Answer to `versionlens/authorization`. */
interface AuthorizationCredential {
	scheme: string;
	token: string;
}

/**
 * Answers the server's credential question from SecretStorage. When nothing
 * is stored for the host, the user is asked once per session.
 */
const answerAuthorization = async (
	state: ExtensionState,
	{ host }: AuthorizationParams,
): Promise<AuthorizationCredential | null> => {
	const stored = await storedCredential(state, host);
	if (stored !== null || state.promptedHosts.has(host)) {
		return stored;
	}
	state.promptedHosts.add(host);
	if (isSuppressed(state, host)) {
		return null;
	}
	// The server names the host only, so the user confirms the full URL
	// (scheme, port and path) in the prompt.
	if (!(await addAuthHeader(state, { authUrl: `https://${host}` }))) {
		return null;
	}
	return storedCredential(state, host);
};

const storedCredential = async (
	state: ExtensionState,
	host: string,
): Promise<AuthorizationCredential | null> => {
	const { context } = state;
	if (context === undefined) {
		return null;
	}
	const candidates = urlAuthenticationEntries(context).filter(
		(entry): boolean => entry.status === noStatus && isHost(entry, host),
	);
	const tokens = await Promise.all(
		candidates.map(async (entry): Promise<string | undefined> => {
			const secretKey = authSecretKey(context, entry.url);
			const token =
				secretKey === undefined
					? undefined
					: await context.secrets.get(secretKey);
			return token;
		}),
	);
	const index = tokens.findIndex(
		(token): boolean => token !== undefined && token !== "",
	);
	const entry = candidates[index];
	const token = tokens[index];
	return entry !== undefined && token !== undefined
		? credential(entry, token)
		: null;
};

/** The server keys hosts lower-case and without a port. */
const isHost = (entry: UrlAuthenticationData, host: string): boolean =>
	URL.canParse(entry.url) &&
	new URL(entry.url).hostname.toLowerCase() === host.toLowerCase();

/** Whether the user already declined to give credentials for the host. */
const isSuppressed = (state: ExtensionState, host: string): boolean =>
	urlAuthenticationEntries(state.context).some(
		(entry): boolean => entry.scheme === notSetScheme && isHost(entry, host),
	);

/**
 * Sends the stored value. Basic is `Basic <base64>`. A custom
 * value is the whole header value, byte for byte, so it goes out with an
 * empty scheme, which the server sends as the token alone.
 */
const credential = (
	entry: UrlAuthenticationData,
	token: string,
): AuthorizationCredential => {
	if (entry.scheme === basicScheme) {
		return { scheme: basicScheme, token };
	}
	return { scheme: "", token };
};

export type { AuthorizationCredential, AuthorizationParams };
export { answerAuthorization };
