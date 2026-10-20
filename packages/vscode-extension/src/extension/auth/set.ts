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
  createUrlAuthenticationData,
  customScheme,
  type DeclinedStatus,
  getUrlAuthentication,
  notSetScheme,
  updateUrlAuthentication,
} from "./store.ts";
import { authorizationValue, confirmInsecureUrl } from "./value.ts";

type AuthContext = NonNullable<ExtensionState["context"]>;

type ProviderQuickPick = QuickPickItem & {
  providerScheme: AuthHeaderMetadata["scheme"];
};

interface AddAuthHeaderOptions {
  authUrl?: string;
  requestUrl?: string;
}

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

  const retried = await retryExistingAuthHeader(context, options);
  if (retried !== undefined) {
    return retried;
  }

  const url = await promptAuthorizationUrl(options);
  if (url === undefined) {
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

  return await storeCredentials(context, url, {
    label: provider.label,
    scheme: provider.providerScheme,
  });
}

/** Asks for the value and stores it with the secret; false when cancelled. */
async function storeCredentials(
  context: AuthContext,
  url: string,
  metadata: AuthHeaderMetadata,
): Promise<boolean> {
  const value = await authorizationValue(metadata.scheme, url);
  if (value.kind === "cancelled") {
    await suppressAuthPrompt(context, url);
    return false;
  }
  const secret = authSecretKey(context, url);
  if (!secret) {
    return false;
  }
  await context.secrets.store(secret, value.value);
  await updateUrlAuthentication(
    context,
    url,
    createUrlAuthenticationData(url, {
      label: metadata.label,
      scheme: metadata.scheme,
    }),
  );
  return true;
}

async function retryExistingAuthHeader(
  context: AuthContext,
  { authUrl }: AddAuthHeaderOptions,
): Promise<boolean | undefined> {
  if (!authUrl) {
    return;
  }
  const header = getUrlAuthentication(context, authUrl);
  if (!header || header.scheme === notSetScheme) {
    return;
  }
  const secret = authSecretKey(context, authUrl);
  const previousValue = secret ? await context.secrets.get(secret) : undefined;
  if (!previousValue) {
    return;
  }
  const retry = await window.showWarningMessage(
    `Could not authenticate credentials with ${authUrl}.\n\nWould you like to re-enter your credentials?`,
    { modal: true },
    "Yes",
  );
  if (retry !== "Yes") {
    await suppressAuthPrompt(context, authUrl, "Credentials failed");
    return false;
  }
  if (header.scheme === basicScheme || previousValue.startsWith("Basic ")) {
    return await storeCredentials(context, authUrl, {
      label: header.label ?? "Basic Auth",
      scheme: basicScheme,
    });
  }
  return await storeCredentials(context, authUrl, {
    label: header.label ?? "Custom Value",
    scheme: customScheme,
  });
}

async function promptAuthorizationUrl(
  options: AddAuthHeaderOptions,
  suggestedUrl = options.authUrl,
): Promise<string | undefined> {
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
  status: DeclinedStatus = "User cancelled",
): Promise<void> {
  if (url) {
    await updateUrlAuthentication(
      context,
      url,
      createUrlAuthenticationData(url, status),
    );
  }
}

export type { AddAuthHeaderOptions };
export { addAuthHeader };
