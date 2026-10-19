import type { ExtensionState } from "../state.ts";
import { addAuthHeader, isAuthHeaderSuppressed } from "./set.ts";
import {
  authSecretKey,
  basicScheme,
  noStatus,
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

const defaultCustomScheme = "Bearer";

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
  const authUrl = `https://${host}`;
  if (isAuthHeaderSuppressed(state, { authUrl })) {
    return null;
  }
  if (!(await addAuthHeader(state, { authUrl }))) {
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

function isHost(entry: UrlAuthenticationData, host: string): boolean {
  return URL.canParse(entry.url) && new URL(entry.url).host === host;
}

/**
 * Basic credentials are stored as the base64 pair. A custom value is the
 * whole header value, so its first word is the scheme and a bare token is
 * a bearer token.
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
  return { scheme: defaultCustomScheme, token };
}

export type { AuthorizationCredential, AuthorizationParams };
export { answerAuthorization };
