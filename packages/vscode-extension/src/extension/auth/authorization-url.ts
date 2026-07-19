import { window } from "#vscode-host";
import { normalizedRegistryUrl } from "./input.ts";

const trailingSlashes = /\/+$/u;

function normalizedAuthorizationUrl(
  value: string | undefined,
): string | undefined {
  const url = normalizedRegistryUrl(value);
  if (!(url && URL.canParse(url))) {
    return;
  }
  return url.replace(trailingSlashes, "");
}

async function validateAuthorizationUrl(
  url: string,
  requestUrl: string | undefined,
): Promise<boolean | undefined> {
  if (requestUrl === undefined) {
    return true;
  }
  const parsedRequestUrl = new URL(requestUrl);
  const parsedAuthUrl = new URL(url);
  if (!sameOrigin(parsedAuthUrl, parsedRequestUrl)) {
    return await retryOrCancel(
      "The authorization url must have the same scheme, domain, and port as the request url",
    );
  }
  if (!pathContains(parsedAuthUrl.pathname, parsedRequestUrl.pathname)) {
    return await retryOrCancel(
      `The authorization url must partially match the request url ${requestUrl}`,
    );
  }
  return true;
}

function sameOrigin(authUrl: URL, requestUrl: URL): boolean {
  return (
    authUrl.protocol === requestUrl.protocol &&
    authUrl.hostname === requestUrl.hostname &&
    effectivePort(authUrl) === effectivePort(requestUrl)
  );
}

function effectivePort(url: URL): string {
  if (url.port) {
    return url.port;
  }
  if (url.protocol === "http:") {
    return "80";
  }
  if (url.protocol === "https:") {
    return "443";
  }
  return "";
}

function pathContains(authPath: string, requestPath: string): boolean {
  const normalizedAuthPath = authPath.replace(trailingSlashes, "");
  const normalizedRequestPath = requestPath.replace(trailingSlashes, "");
  return (
    normalizedAuthPath === "" ||
    normalizedRequestPath === normalizedAuthPath ||
    normalizedRequestPath.startsWith(`${normalizedAuthPath}/`)
  );
}

async function retryOrCancel(message: string): Promise<false | undefined> {
  let result: false | undefined;
  if (await promptRetry(message)) {
    result = false;
  }
  return result;
}

async function promptRetry(message: string): Promise<boolean> {
  const choice = await window.showInformationMessage(
    message,
    { modal: true, detail: "" },
    "Retry",
  );
  return Boolean(choice);
}

export { normalizedAuthorizationUrl, validateAuthorizationUrl };
