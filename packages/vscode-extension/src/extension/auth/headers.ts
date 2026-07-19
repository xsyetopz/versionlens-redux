import { optionalProperty } from "../config/optional.ts";
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

interface ResolvedAuthHeader {
  name: string;
  url?: string;
  value: string;
}

async function authHeaders(state: ExtensionState): Promise<NativeHttpHeader[]> {
  const { context } = state;
  if (!context) {
    return [];
  }

  const headers = await Promise.all(
    urlAuthenticationEntries(context).map(
      (entry): Promise<ResolvedAuthHeader | undefined> =>
        urlAuthenticationHeader(context, entry),
    ),
  );
  return headers
    .filter((header): header is ResolvedAuthHeader => header !== undefined)
    .map((header): { value: string; url?: string; name: string } => ({
      name: header.name,
      ...optionalProperty("url", header.url),
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
    return;
  }

  const secretKey = authSecretKey(context, entry.url);
  let token: string | undefined;
  if (secretKey) {
    token = await context.secrets.get(secretKey);
  }
  if (!token) {
    return;
  }

  return {
    name: authorizationHeaderName,
    url: entry.url,
    value: authHeaderValue(entry.scheme, token),
  };
}

function authHeaderValue(scheme: string, token: string): string {
  if (scheme === basicScheme) {
    return `${basicScheme} ${token}`;
  }
  return token;
}

export { authHeaders };
