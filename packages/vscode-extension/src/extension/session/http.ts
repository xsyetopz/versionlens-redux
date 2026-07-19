import { type Uri, workspace } from "#vscode-host";
import { authHeaders } from "../auth/headers.ts";
import { optionalProperty } from "../config/optional.ts";
import type { NativeHttpConfig, NativeHttpHeader } from "../native/config.ts";
import type { ExtensionState } from "../state.ts";
import { configuredValue } from "./configured.ts";

export async function httpConfig(
  state: ExtensionState,
  resource?: Uri,
): Promise<NativeHttpConfig> {
  const http = workspace.getConfiguration("http", resource);
  const versionlens = workspace.getConfiguration("versionlens", resource);
  const proxy = configuredValue<string | undefined>("proxy", http);
  const strictSsl = configuredValue<boolean | null | undefined>(
    "http.strictSSL",
    versionlens,
  );
  const headers = await authHeaders(state);

  const configuredStrictSsl = strictSsl ?? undefined;
  let configuredHeaders: NativeHttpHeader[] | undefined;
  if (headers.length > 0) {
    configuredHeaders = headers;
  }
  return {
    ...optionalProperty("authHeaders", configuredHeaders),
    ...optionalProperty("proxy", proxy),
    ...optionalProperty("strictSsl", configuredStrictSsl),
  };
}
