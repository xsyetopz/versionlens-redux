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
async function answerAuthorization(
  state: ExtensionState,
  { host }: AuthorizationParams,
): Promise<AuthorizationCredential | null> {
  const stored = await storedCredential(state, host);
  if (stored || state.promptedHosts.has(host)) {
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
  return await storedCredential(state, host);
}

async function storedCredential(
  state: ExtensionState,
  host: string,
): Promise<AuthorizationCredential | null> {
  const { context } = state;
  if (!context) {
    return null;
  }
  for (const entry of urlAuthenticationEntries(context)) {
    if (entry.status !== noStatus || !isHost(entry, host)) {
      continue;
    }
    const secretKey = authSecretKey(context, entry.url);
    const token = secretKey ? await context.secrets.get(secretKey) : undefined;
    if (token) {
      return credential(entry, token);
    }
  }
  return null;
}

/** The server keys hosts lower-case and without a port. */
function isHost(entry: UrlAuthenticationData, host: string): boolean {
  return (
    URL.canParse(entry.url) &&
    new URL(entry.url).hostname.toLowerCase() === host.toLowerCase()
  );
}

/** Whether the user already declined to give credentials for the host. */
function isSuppressed(state: ExtensionState, host: string): boolean {
  return urlAuthenticationEntries(state.context).some(
    (entry): boolean => entry.scheme === notSetScheme && isHost(entry, host),
  );
}

/**
 * Basic credentials are stored as the base64 pair. A custom value is the
 * whole header value, so its first word is the scheme. A bare token has an
 * empty scheme, which the server sends as the whole header value.
 */
function credential(
  entry: UrlAuthenticationData,
  token: string,
): AuthorizationCredential | null {
  if (entry.scheme === basicScheme) {
    return { scheme: basicScheme, token };
  }
  const separator = token.indexOf(" ");
  if (separator > 0) {
    return {
      scheme: token.slice(0, separator),
      token: token.slice(separator + 1).trim(),
    };
  }
  return { scheme: "", token };
}

export type { AuthorizationCredential, AuthorizationParams };
export { answerAuthorization };
