import {
  SuggestionCategory,
  SuggestionStatusText,
  SuggestionTypes
} from '#domain/packages';
import type { KeyDictionary } from '#domain/utils';
import { SuggestionCommandFeatures } from '#extension';
import { SuggestionCodeLens } from '#extension/suggestions';
import * as os from 'node:os';

const isWindows = os.type() === "Windows_NT";

/**
 * Creates a status-only command (non-clickable).
 * @param status The status text to display.
 * @param codeLens The code lens to update.
 * @returns The updated code lens.
 */
export function createStatusCommand(status: string, codeLens: SuggestionCodeLens) {
  return codeLens.setCommand(status, "");
}

/**
 * Creates a command that triggers a dependency update when clicked.
 * @param title The command title.
 * @param codeLens The code lens to update.
 * @returns The updated code lens.
 */
export function createUpdateableCommand(title: string, codeLens: SuggestionCodeLens) {
  return codeLens.setCommand(
    title,
    SuggestionCommandFeatures.OnUpdateDependencyClick,
    [codeLens]
  );
}

/**
 * Creates an "Invalid" status command.
 * @param codeLens The code lens to update.
 * @returns The updated code lens.
 */
export function createInvalidCommand(codeLens: SuggestionCodeLens) {
  return codeLens.setCommand(SuggestionStatusText.Invalid, "");
}

/**
 * Creates a command that opens a directory link when clicked.
 * @param title The command title.
 * @param codeLens The code lens to update.
 * @returns The updated code lens.
 */
export function createDirectoryLinkCommand(title: string, codeLens: SuggestionCodeLens) {
  const cmd = SuggestionCommandFeatures.OnFileLinkClick as string;
  return codeLens.setCommand(title, cmd, [codeLens]);
}

/**
 * Creates a command that opens a build selection prompt when clicked.
 * @param title The command title.
 * @param codeLens The code lens to update.
 * @returns The updated code lens.
 */
export function createChooseBuildCommand(title: string, codeLens: SuggestionCodeLens) {
  const cmd = SuggestionCommandFeatures.OnChooseBuildClick as string;
  return codeLens.setCommand(title, cmd, [codeLens]);
}

/**
 * Evaluates a suggestion and assigns the appropriate command to the code lens based on its category.
 * @param codeLens The code lens to resolve.
 * @param indicators Map of indicators for each suggestion category.
 */
export function createSuggestedVersionCommand(
  codeLens: SuggestionCodeLens,
  indicators: KeyDictionary<string>
) {
  if (!codeLens.packageResponse.suggestion) return createInvalidCommand(codeLens);

  const { name, version, category, type } = codeLens.packageResponse.suggestion;

  // get the category indicator
  const indicator = indicators[category] + (isWindows ? '' : ' ');
  const indicatedName = indicator ? `${indicator}${name}` : name;

  // create the indicated command title
  const cmdTitle = type === SuggestionTypes.tag
    ? indicatedName.trim()
    : `${indicatedName} ${version}`.trim();

  // create the suggestion command
  switch (category) {
    case SuggestionCategory.Updateable:
      createUpdateableCommand(cmdTitle, codeLens);
      break;

    case SuggestionCategory.Directory:
      const fileTitle = `${indicatedName}${version}`.trim();
      createDirectoryLinkCommand(fileTitle, codeLens);
      break;

    case SuggestionCategory.Build:
      createChooseBuildCommand(indicatedName, codeLens);
      break;

    default:
      createStatusCommand(cmdTitle, codeLens);
      break;
  }
}