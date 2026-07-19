import {
  type ExtensionContext,
  type QuickPickItem,
  window,
} from "#vscode-host";
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
  const choices: AuthHeaderQuickPick[] = urlAuthenticationChoices(context);
  const picked = await window.showQuickPick<AuthHeaderQuickPick>(choices, {
    canPickMany: true,
    placeHolder: "Choose which urls to remove",
    title: "Clear url authentication",
  });
  const pickedItems = picked ?? [];
  if (!(context && pickedItems.length > 0)) {
    return false;
  }

  await Promise.all(
    pickedItems.map(async (item): Promise<void> => {
      await removeUrlAuthentication(context, item.entry.url);
      if (item.entry.scheme !== notSetScheme) {
        const secret = authSecretKey(context, item.entry.url);
        if (secret) {
          await context.secrets.delete(secret);
        }
      }
    }),
  );
  return true;
}

type AuthenticationDetail = { detail: string } | { detail?: never };

function urlAuthenticationChoices(
  context: ExtensionContext | undefined,
): AuthHeaderQuickPick[] {
  return urlAuthenticationEntries(context).map(
    (
      entry,
    ):
      | { detail: string; entry: UrlAuthenticationData; label: string }
      | { detail?: never; entry: UrlAuthenticationData; label: string } => ({
      entry,
      label: entry.url,
      ...urlAuthenticationDetail(entry),
    }),
  );
}

function urlAuthenticationDetail(
  entry: UrlAuthenticationData,
): AuthenticationDetail {
  const detail: string[] = [];
  if (entry.scheme !== notSetScheme) {
    if (entry.protocol === "http:") {
      detail.push("Unsecured");
    } else {
      detail.push("Secure");
    }
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
