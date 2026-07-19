import { window } from "#vscode-host";
import { normalizedRequiredInput } from "./input.ts";
import { customScheme } from "./store.ts";

type AuthenticationScheme = "Basic" | "Custom";
type AuthorizationPromptResult = Promise<string | undefined>;

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
  scheme: AuthenticationScheme,
  url: string,
): AuthorizationPromptResult {
  if (scheme === customScheme) {
    return normalizedRequiredInput(
      await window.showInputBox({
        ignoreFocusOut: true,
        password: true,
        placeHolder: "Authorization value",
        prompt: `Enter the authorization value for ${url}`,
      }),
    );
  }
  return await basicAuthorizationValue(url);
}

async function basicAuthorizationValue(url: string): AuthorizationPromptResult {
  const username = await window.showInputBox({
    ignoreFocusOut: true,
    password: false,
    placeHolder: "Basic auth username",
    prompt: `Enter the basic auth username for ${url}`,
  });
  if (username === undefined) {
    return;
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
    return;
  }
  const password = await window.showInputBox({
    ignoreFocusOut: true,
    password: true,
    placeHolder: "Basic auth password",
    prompt: `Enter the basic auth password for ${url}`,
  });
  if (password === undefined) {
    return;
  }
  return Buffer.from(`${username}:${password}`).toString("base64");
}

export type { AuthenticationScheme, AuthorizationPromptResult };
export { authorizationValue, confirmInsecureUrl };
