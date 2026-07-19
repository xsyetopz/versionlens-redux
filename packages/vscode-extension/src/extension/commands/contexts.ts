import { commands } from "#vscode-host";
import { analyzeDocument } from "../diagnostics/analyze.ts";
import { activeFileDocument } from "../documents/file.ts";
import type { ExtensionState } from "../state.ts";
import { customInstallTaskLabel } from "../tasks/custom-install.ts";

export async function updateContexts(
  state: ExtensionState,
): Promise<string | false> {
  const activeDocument = activeFileDocument();
  let activeOutput: ReturnType<typeof analyzeDocument>;
  if (activeDocument) {
    activeOutput = analyzeDocument(state, activeDocument);
  }
  const providerActive = activeOutput?.activeProviderName ?? false;

  await commands.executeCommand(
    "setContext",
    "versionlens.show",
    state.flags.showVersionLenses,
  );
  await commands.executeCommand(
    "setContext",
    "versionlens.showPrereleases",
    state.flags.showPrereleases,
  );
  await commands.executeCommand(
    "setContext",
    "versionlens.showOutdated",
    state.flags.showOutdated,
  );
  await commands.executeCommand(
    "setContext",
    "versionlens.providerActive",
    providerActive,
  );
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
  await commands.executeCommand(
    "setContext",
    "versionlens.showCustomInstall",
    providerActive && Boolean(customInstallTaskLabel(state, activeDocument)),
  );
  await commands.executeCommand(
    "setContext",
    "versionlens.showSortAlphabetically",
    providerActive && (activeOutput?.canSortDependencies ?? false),
  );

  return providerActive;
}
