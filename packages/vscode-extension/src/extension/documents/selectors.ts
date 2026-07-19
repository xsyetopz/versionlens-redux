import {
  type DocumentFilter,
  type DocumentSelector,
  type Uri,
  workspace,
} from "#vscode-host";
import { enabledFilePatternKeys } from "../config/keys/files.ts";

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

export { documentSelectors };
