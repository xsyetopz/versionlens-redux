import { type QuickPickItem, window } from "#vscode-host";
import type { ExtensionState } from "../state.ts";
import {
  authSecretKey,
  noStatus,
  notSetScheme,
  removeUrlAuthentication,
  type UrlAuthenticationData,
  urlAuthenticationEntries,
} from "./store.ts";

type AuthHeaderQuickPick = QuickPickItem & {
  entry: UrlAuthenticationData;
};

async function removeAuthHeader(state: ExtensionState): Promise<boolean> {
  const { context } = state;
  const choices = urlAuthenticationEntries(context).map(
    (entry): AuthHeaderQuickPick => ({
      entry,
      label: entry.url,
      ...urlAuthenticationDetail(entry),
    }),
  );
  const picked = await window.showQuickPick<AuthHeaderQuickPick>(choices, {
    canPickMany: true,
    placeHolder: "Choose which urls to remove",
    title: "Clear url authentication",
  });
  if (!(context && picked && picked.length > 0)) {
    return false;
  }

  await Promise.all(
    picked.map(async ({ entry }): Promise<void> => {
      await removeUrlAuthentication(context, entry.url);
      const secret = authSecretKey(context, entry.url);
      if (entry.scheme !== notSetScheme && secret) {
        await context.secrets.delete(secret);
      }
    }),
  );
  return true;
}

function urlAuthenticationDetail(
  entry: UrlAuthenticationData,
): { detail: string } | { detail?: never } {
  const detail: string[] = [];
  if (entry.scheme !== notSetScheme) {
    detail.push(entry.protocol === "http:" ? "Unsecured" : "Secure");
    if (entry.label) {
      detail.push(entry.label);
    }
  }
  if (entry.status !== noStatus) {
    detail.push(`(${entry.status})`);
  }
  if (detail.length > 0) {
    return { detail: detail.join(" ") };
  }
  return {};
}

export { removeAuthHeader };
