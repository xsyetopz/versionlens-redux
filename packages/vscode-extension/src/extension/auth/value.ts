import { window } from "#vscode-host";
import { normalizedRequiredInput } from "./authorization-url.ts";
import { type AuthHeaderMetadata, customScheme } from "./store.ts";

type AuthorizationValue =
  | { kind: "cancelled" }
  | { kind: "provided"; value: string };

const cancelled: AuthorizationValue = { kind: "cancelled" };

function provided(value: string): AuthorizationValue {
  return { kind: "provided", value };
}

async function confirmInsecureUrl(url: string): Promise<boolean> {
  if (url.startsWith("https:")) {
    return true;
  }
  const choice = await window.showWarningMessage(
    `${url} is using the unsecure HTTP protocol.\n\nAre you sure you want to send your credentials with this url?`,
    { modal: true },
    "Yes",
  );
  return choice === "Yes";
}

async function authorizationValue(
  scheme: AuthHeaderMetadata["scheme"],
  url: string,
): Promise<AuthorizationValue> {
  if (scheme === customScheme) {
    const value = normalizedRequiredInput(
      await window.showInputBox({
        ignoreFocusOut: true,
        password: true,
        placeHolder: "Authorization value",
        prompt: `Enter the authorization value for ${url}`,
      }),
    );
    if (value === undefined) {
      return cancelled;
    }
    return provided(value);
  }
  return await basicAuthorizationValue(url);
}

async function basicAuthorizationValue(
  url: string,
): Promise<AuthorizationValue> {
  const username = await window.showInputBox({
    ignoreFocusOut: true,
    password: false,
    placeHolder: "Basic auth username",
    prompt: `Enter the basic auth username for ${url}`,
  });
  if (username === undefined) {
    return cancelled;
  }
  if (username.includes(":")) {
    const retry = await window.showInformationMessage(
      "You cannot have a ':' character in the user name.\n\nDo you want re-enter the username or cancel?",
      { modal: true },
      "Retry",
    );
    if (retry) {
      return basicAuthorizationValue(url);
    }
    return cancelled;
  }
  const password = await window.showInputBox({
    ignoreFocusOut: true,
    password: true,
    placeHolder: "Basic auth password",
    prompt: `Enter the basic auth password for ${url}`,
  });
  if (password === undefined) {
    return cancelled;
  }
  return provided(Buffer.from(`${username}:${password}`).toString("base64"));
}

export { authorizationValue, confirmInsecureUrl };
