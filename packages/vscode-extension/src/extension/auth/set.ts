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
  createEmptyUrlAuthenticationData,
  createUrlAuthenticationData,
  customScheme,
  getUrlAuthentication,
  notSetScheme,
  type UrlAuthenticationData,
  updateUrlAuthentication,
} from "./store.ts";
import {
  type AuthenticationScheme,
  type AuthorizationPromptResult,
  authorizationValue,
  confirmInsecureUrl,
} from "./value.ts";

type AuthContext = NonNullable<ExtensionState["context"]>;
type AuthHeaderStatus = "User cancelled" | "Credentials failed";

type ProviderQuickPick = QuickPickItem & {
  providerScheme: AuthenticationScheme;
};

interface AddAuthHeaderOptions {
  authUrl?: string;
  requestUrl?: string;
}

const userCancelledStatus = "User cancelled";
const credentialsFailedStatus = "Credentials failed";
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

async function addAuthHeader(
  state: ExtensionState,
  options: AddAuthHeaderOptions = {},
): Promise<boolean> {
  const { context } = state;
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

  const provider = await window.showQuickPick(authenticationProviders, {
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
): Promise<boolean | undefined> {
  const { context } = state;
  if (!(context && options.authUrl)) {
    return;
  }
  const header = getUrlAuthentication(context, options.authUrl);
  if (!header || header.scheme === notSetScheme) {
    return;
  }
  const secret = authSecretKey(context, options.authUrl);
  let previousValue: string | undefined;
  if (secret) {
    previousValue = await context.secrets.get(secret);
  }
  if (!previousValue) {
    return;
  }
  const retry = await window.showWarningMessage(
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
  if (header.scheme === basicScheme) {
    return basicScheme;
  }
  return authSchemeForValue(value);
}

function authSchemeForValue(value: string): AuthenticationScheme {
  if (value.startsWith(`${basicScheme} `)) {
    return basicScheme;
  }
  return customScheme;
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

function authHeaderLabelForScheme(
  scheme: AuthenticationScheme,
): "Basic Auth" | "Custom Value" {
  if (scheme === basicScheme) {
    return "Basic Auth";
  }
  return "Custom Value";
}

function isAuthHeaderSuppressed(
  state: ExtensionState,
  options: AddAuthHeaderOptions | undefined,
): boolean {
  const authUrl = options?.authUrl;
  const { context } = state;
  if (!(authUrl && context)) {
    return false;
  }
  return getUrlAuthentication(context, authUrl)?.scheme === notSetScheme;
}

async function promptAuthorizationUrl(
  options: AddAuthHeaderOptions,
  suggestedUrl = options.authUrl,
): AuthorizationPromptResult {
  const url = normalizedAuthorizationUrl(
    await window.showInputBox({
      ignoreFocusOut: true,
      placeHolder: "Authorization url",
      prompt: "Enter the authorization url for package requests",
      ...optionalProperty("value", suggestedUrl),
    }),
  );
  if (!url) {
    return;
  }
  const validation = await validateAuthorizationUrl(url, options.requestUrl);
  if (validation === undefined) {
    return;
  }
  if (!validation) {
    return promptAuthorizationUrl(options, url);
  }
  return url;
}

async function suppressAuthPrompt(
  context: AuthContext,
  url: string | undefined,
  status: AuthHeaderStatus = userCancelledStatus,
): Promise<void> {
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
  context: AuthContext,
  url: string,
  metadataOrStatus?: AuthHeaderMetadata | AuthHeaderStatus,
): Promise<void> {
  let value: UrlAuthenticationData;
  if (typeof metadataOrStatus === "string") {
    value = createEmptyUrlAuthenticationData(url, metadataOrStatus);
  } else {
    value = createUrlAuthenticationData(
      url,
      metadataOrStatus ?? { label: "Custom Value", scheme: customScheme },
    );
  }
  await updateUrlAuthentication(context, url, value);
}

export type { AddAuthHeaderOptions };
export { addAuthHeader, isAuthHeaderSuppressed };
