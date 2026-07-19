import { type TextDocument, workspace } from "#vscode-host";
import { optionalProperty } from "../config/optional.ts";
import type { NativeDocumentInput } from "../native/input.ts";

export function documentInput(document: TextDocument): NativeDocumentInput {
  const workspaceRoot = workspace.getWorkspaceFolder(document.uri)?.uri.fsPath;

  return {
    uri: document.uri.toString(),
    languageId: document.languageId,
    text: document.getText(),
    ...optionalProperty("workspaceRoot", workspaceRoot),
  };
}
