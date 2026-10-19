import {
  commands,
  type ExtensionContext,
  extensions,
  window,
  workspace,
} from "#vscode-host";
import { startClient } from "../client.ts";
import { updateContexts } from "../commands/contexts.ts";
import { reloadConfigurationState } from "../session/flags.ts";
import type { ExtensionState } from "../state.ts";
import {
  registerExtensionSubscriptions,
  trackDocument,
} from "./subscriptions.ts";
import { initializeUi } from "./ui.ts";

async function activateExtension(
  state: ExtensionState,
  context: ExtensionContext,
): Promise<void> {
  state.context = context;
  reloadConfigurationState(state);
  initializeUi(state);
  await warnWhenOriginalVersionLensInstalled(state);
  warnWhenCodeLensesDisabled(state);
  registerExtensionSubscriptions(state, context);

  await startClient(state);
  await Promise.all(
    workspace.textDocuments.map(
      (document): Promise<void> => trackDocument(state, document),
    ),
  );
  await updateContexts(state);
}

async function warnWhenOriginalVersionLensInstalled(
  state: ExtensionState,
): Promise<void> {
  const originalExtensionId = "pflannery.vscode-versionlens";
  if (!extensions.getExtension(originalExtensionId)) {
    return;
  }

  const disableAction = "Disable original VersionLens";
  const message =
    "VersionLens Redux conflicts with the original VersionLens extension. Disable the original extension before using VersionLens Redux in this workspace.";
  state.ui.outputChannel?.appendLine(message);

  const selected = await window.showWarningMessage(message, disableAction);
  if (selected !== disableAction) {
    return;
  }

  await commands.executeCommand(
    "workbench.extensions.action.disableExtension",
    originalExtensionId,
  );
}

function warnWhenCodeLensesDisabled(state: ExtensionState): void {
  const editorCodeLensKey = "editor.codeLens";
  if (
    workspace.getConfiguration().get<boolean>(editorCodeLensKey, true) !== false
  ) {
    return;
  }

  state.ui.outputChannel?.appendLine(
    "Code lenses are disabled. This extension won't work unless you enable 'editor.codeLens' in your vscode settings",
  );
}

export { activateExtension };
