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

function storeEntries(context: ExtensionContext): [string, unknown][] {
  const collection = context.workspaceState?.get<unknown>(storeKey, {});
  if (typeof collection === "object" && collection !== null) {
    return Object.entries(collection);
  }
  return [];
}

function urlAuthenticationEntries(
  context: ExtensionContext | undefined,
): UrlAuthenticationData[] {
  if (!context) {
    return [];
  }
  return storeEntries(context)
    .map((entry): unknown => entry[1])
    .filter(isUrlAuthenticationData);
}

function getUrlAuthentication(
  context: ExtensionContext | undefined,
  url: string,
): UrlAuthenticationData | undefined {
  return urlAuthenticationEntries(context).find(
    (entry): boolean => entry.url === url,
  );
}

async function updateUrlAuthentication(
  context: ExtensionContext,
  url: string,
  value: UrlAuthenticationData,
): Promise<void> {
  const entries = storeEntries(context).filter(([key]): boolean => key !== url);
  await context.workspaceState.update(
    storeKey,
    Object.fromEntries([...entries, [url, value]]),
  );
}

async function removeUrlAuthentication(
  context: ExtensionContext,
  url: string,
): Promise<void> {
  await context.workspaceState.update(
    storeKey,
    Object.fromEntries(
      storeEntries(context).filter(([key]): boolean => key !== url),
    ),
  );
}

/** Stored data for a url: credentials for metadata, a decline for a status. */
function createUrlAuthenticationData(
  url: string,
  metadata: AuthHeaderMetadata | DeclinedStatus,
): UrlAuthenticationData {
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
}

function authSecretKey(
  context: ExtensionContext,
  url: string,
): string | undefined {
  let folder = context.storageUri?.path;
  const activePath = window?.activeTextEditor?.document.uri.path;
  if (!folder && activePath) {
    folder = dirname(activePath);
  }
  if (folder) {
    return `${folder}__${url}`;
  }
  return;
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
