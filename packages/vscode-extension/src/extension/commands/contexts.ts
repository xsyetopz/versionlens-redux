import { commands, window } from "#vscode-host";
import { fileDocument } from "../documents.ts";
import { manifestInfo } from "../manifest.ts";
import type { ExtensionState } from "../state.ts";
import { customInstallTaskLabel } from "../tasks/custom-install.ts";

export async function updateContexts(
  state: ExtensionState,
): Promise<string | false> {
  const activeDocument = fileDocument(window.activeTextEditor?.document);
  let info: Awaited<ReturnType<typeof manifestInfo>>;
  if (activeDocument) {
    info = await manifestInfo(state, activeDocument);
  }
  const providerActive = (info?.supported && info.provider) || false;
  await setContexts(state, {
    "versionlens.providerActive": providerActive,
    "versionlens.show": state.flags.showVersionLenses,
    "versionlens.showCustomInstall":
      providerActive &&
      Boolean(await customInstallTaskLabel(state, activeDocument)),
    "versionlens.showOutdated": state.flags.showOutdated,
    "versionlens.showPrereleases": state.flags.showPrereleases,
    "versionlens.showSortAlphabetically":
      providerActive && (info?.canSortDependencies ?? false),
  });
  return providerActive;
}

async function setContexts(
  state: ExtensionState,
  contexts: Record<string, boolean | string>,
): Promise<void> {
  for (const [key, value] of Object.entries(contexts)) {
    await commands.executeCommand("setContext", key, value);
  }
  await commands.executeCommand(
    "setContext",
    "versionlens.providerBusy",
    state.flags.providerBusy,
  );
  await commands.executeCommand(
    "setContext",
    "versionlens.providerError",
    state.flags.providerError,
  );
}
