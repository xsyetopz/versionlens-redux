import { window, workspace } from "#vscode-host";
import { fileDocument } from "../documents.ts";
import { manifestInfo } from "../manifest.ts";
import type { ExtensionState } from "../state.ts";
import { runTask } from "./runner.ts";

export async function runCustomInstall(state: ExtensionState): Promise<void> {
  const document = fileDocument(window.activeTextEditor?.document);
  if (!document) {
    return;
  }
  const label = await customInstallTaskLabel(state, document);
  if (!label) {
    return;
  }

  await runTask(label, workspace.getWorkspaceFolder(document.uri));
}

export async function customInstallTaskLabel(
  state: ExtensionState,
  document = window.activeTextEditor?.document,
): Promise<string | undefined> {
  const file = fileDocument(document);
  let key: string | undefined;
  let label: string | undefined;
  if (file) {
    key = (await manifestInfo(state, file))?.installTaskConfigKey ?? undefined;
  }
  if (key) {
    label = workspace
      .getConfiguration("versionlens")
      .get<string | undefined>(key);
  }
  return label;
}
