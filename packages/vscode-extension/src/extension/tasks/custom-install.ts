import { window, workspace } from "#vscode-host";
import { analyzeDocument } from "../diagnostics/analyze.ts";
import { activeFileDocument, fileDocument } from "../documents/file.ts";
import type { ExtensionState } from "../state.ts";
import { runTask } from "./runner.ts";

export async function runCustomInstall(state: ExtensionState): Promise<void> {
  const document = activeFileDocument();
  if (!document) {
    return;
  }
  const label = customInstallTaskLabel(state, document);
  if (!label) {
    return;
  }

  await runTask(label, workspace.getWorkspaceFolder(document.uri));
}

export function customInstallTaskLabel(
  state: ExtensionState,
  document = window.activeTextEditor?.document,
): string | undefined {
  const file = fileDocument(document);
  let key: string | undefined;
  let label: string | undefined;
  if (file) {
    key = analyzeDocument(state, file)?.installTaskConfigKey;
  }
  if (key) {
    label = workspace
      .getConfiguration("versionlens")
      .get<string | undefined>(key);
  }
  return label;
}
