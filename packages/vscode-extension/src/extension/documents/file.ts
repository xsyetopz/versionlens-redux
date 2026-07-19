import { type TextDocument, window } from "#vscode-host";

export function activeFileDocument(): TextDocument | undefined {
  return fileDocument(window.activeTextEditor?.document);
}

export function fileDocument(
  document: TextDocument | undefined,
): TextDocument | undefined {
  let file: TextDocument | undefined;
  if (document?.uri.scheme === "file") {
    file = document;
  }
  return file;
}
