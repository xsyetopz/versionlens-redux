import { dirname } from "node:path";
import { type ExtensionContext, window } from "#vscode-host";

type AuthenticationScheme = "NotSet" | "Basic" | "Custom";
type UrlAuthenticationStatus =
  | "NoStatus"
  | "User cancelled"
  | "Credentials failed";

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

const urlAuthenticationStoreKey = "UrlAuthenticationStore";
const authorizationHeaderName = "Authorization";
const noStatus = "NoStatus";
const notSetScheme = "NotSet";
const basicScheme = "Basic";
const customScheme = "Custom";

function urlAuthenticationEntries(
  context: ExtensionContext | undefined,
): UrlAuthenticationData[] {
  if (!context) {
    return [];
  }
  return collectionValues(context).filter(isUrlAuthenticationData);
}

function getUrlAuthentication(
  context: ExtensionContext | undefined,
  url: string,
): UrlAuthenticationData | undefined {
  let authentication: UrlAuthenticationData | undefined;
  if (context) {
    const entry = collectionEntries(context).find(
      ([key]): boolean => key === url,
    )?.[1];
    if (isUrlAuthenticationData(entry)) {
      authentication = entry;
    }
  }
  return authentication;
}

async function updateUrlAuthentication(
  context: ExtensionContext,
  url: string,
  value: UrlAuthenticationData,
): Promise<void> {
  const entries = collectionEntries(context).filter(
    ([key]): boolean => key !== url,
  );
  await context.workspaceState.update(
    urlAuthenticationStoreKey,
    Object.fromEntries([...entries, [url, value]]),
  );
}

async function removeUrlAuthentication(
  context: ExtensionContext,
  url: string,
): Promise<void> {
  await context.workspaceState.update(
    urlAuthenticationStoreKey,
    Object.fromEntries(
      collectionEntries(context).filter(([key]): boolean => key !== url),
    ),
  );
}

function createUrlAuthenticationData(
  url: string,
  metadata: AuthHeaderMetadata,
): UrlAuthenticationData {
  return {
    label: metadata.label,
    protocol: new URL(url).protocol,
    scheme: metadata.scheme,
    status: noStatus,
    url,
  };
}

function createEmptyUrlAuthenticationData(
  url: string,
  status: Exclude<UrlAuthenticationStatus, "NoStatus"> = "User cancelled",
): UrlAuthenticationData {
  return {
    protocol: new URL(url).protocol,
    scheme: notSetScheme,
    status,
    url,
  };
}

function authSecretKey(
  context: ExtensionContext,
  url: string,
): string | undefined {
  const resourceFolderPath = resourceFolderPathForAuth(context);
  let secretKey: string | undefined;
  if (resourceFolderPath) {
    secretKey = `${resourceFolderPath}__${url}`;
  }
  return secretKey;
}

function resourceFolderPathForAuth(
  context: ExtensionContext,
): string | undefined {
  let resourcePath: string | undefined;
  if (context.storageUri?.path) {
    resourcePath = context.storageUri.path;
  } else {
    const activePath = window?.activeTextEditor?.document.uri.path;
    if (activePath) {
      resourcePath = dirname(activePath);
    }
  }
  return resourcePath;
}

function collectionEntries(context: ExtensionContext): [string, unknown][] {
  const { workspaceState } = context;
  if (!workspaceState) {
    return [];
  }
  const collection = workspaceState.get<unknown>(urlAuthenticationStoreKey, {});
  if (typeof collection === "object" && collection !== null) {
    return Object.entries(collection);
  }
  return [];
}

function collectionValues(context: ExtensionContext): unknown[] {
  return collectionEntries(context).map((entry): unknown => entry[1]);
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

export type { AuthHeaderMetadata, UrlAuthenticationData };
export {
  authorizationHeaderName,
  authSecretKey,
  basicScheme,
  createEmptyUrlAuthenticationData,
  createUrlAuthenticationData,
  customScheme,
  getUrlAuthentication,
  noStatus,
  notSetScheme,
  removeUrlAuthentication,
  resourceFolderPathForAuth,
  updateUrlAuthentication,
  urlAuthenticationEntries,
  urlAuthenticationStoreKey,
};
