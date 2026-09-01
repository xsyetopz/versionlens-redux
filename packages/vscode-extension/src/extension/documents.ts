import {
  type DocumentFilter,
  type DocumentSelector,
  Range,
  type TextDocument,
  type Uri,
  workspace,
} from "#vscode-host";
import { enabledFilePatternKeys } from "./config/keys.ts";
import { optionalProperty } from "./config/optional.ts";
import type { NativeDocumentInput, NativeRange } from "./native.ts";

function fileDocument(
  document: TextDocument | undefined,
): TextDocument | undefined {
  let file: TextDocument | undefined;
  if (document?.uri.scheme === "file") {
    file = document;
  }
  return file;
}

function documentInput(document: TextDocument): NativeDocumentInput {
  const workspaceRoot = workspace.getWorkspaceFolder(document.uri)?.uri.fsPath;

  return {
    uri: document.uri.toString(),
    languageId: document.languageId,
    text: document.getText(),
    version: document.version,
    ...optionalProperty("workspaceRoot", workspaceRoot),
  };
}

function toRange(range: NativeRange): Range {
  return new Range(
    range.start.line,
    range.start.character,
    range.end.line,
    range.end.character,
  );
}

type FileDocumentSelectors = Array<{
  language: string;
  pattern: string;
  scheme: string;
}>;

function documentSelectors(): DocumentSelector {
  const selectors = [
    undefined,
    ...(workspace.workspaceFolders ?? []).map(({ uri }): Uri => uri),
  ].flatMap(
    (resource): Array<{ language: string; pattern: string; scheme: string }> =>
      selectorsForResource(resource),
  );
  return [
    ...new Map(
      selectors.map(
        (
          selector,
        ): [string, { language: string; pattern: string; scheme: string }] => [
          selectorKey(selector),
          selector,
        ],
      ),
    ).values(),
  ];
}

function selectorsForResource(
  resource: Uri | undefined,
): FileDocumentSelectors {
  const config = workspace.getConfiguration("versionlens", resource);
  return enabledFilePatternKeys(
    config.get<string[]>("enabledProviders"),
  ).flatMap(
    ([, key, languages]): Array<{
      language: string;
      pattern: string;
      scheme: string;
    }> => {
      const pattern = config.get<string>(key) ?? "**/*";
      return languages.map(
        (language): { language: string; pattern: string; scheme: string } => ({
          language,
          pattern,
          scheme: "file",
        }),
      );
    },
  );
}

function selectorKey(selector: DocumentFilter): string {
  return `${selector.scheme}\0${selector.language}\0${String(selector.pattern)}`;
}

export { documentInput, documentSelectors, fileDocument, toRange };
