import {
  SuggestionCategory,
  SuggestionStatusText,
  SuggestionTypes
} from 'domain/packages';
import { KeyDictionary } from 'domain/utils';
import * as os from 'node:os';
import {
  SuggestionCodeLens,
  SuggestionCommandFeatures
} from 'presentation.extension';

const isWindows = os.type() === "Windows_NT";

export function createStatusCommand(status: string, codeLens: SuggestionCodeLens) {
  return codeLens.setCommand(status, "");
}

export function createUpdateableCommand(title: string, codeLens: SuggestionCodeLens) {
  return codeLens.setCommand(
    title,
    SuggestionCommandFeatures.OnUpdateDependencyClick,
    [codeLens]
  );
}

export function createInvalidCommand(codeLens: SuggestionCodeLens) {
  return codeLens.setCommand(SuggestionStatusText.Invalid, "");
}

export function createDirectoryLinkCommand(title: string, codeLens: SuggestionCodeLens) {
  const cmd = SuggestionCommandFeatures.OnFileLinkClick as string;
  return codeLens.setCommand(title, cmd, [codeLens]);
}

export function createSuggestedVersionCommand(
  codeLens: SuggestionCodeLens,
  indicators: KeyDictionary<string>
) {
  if (!codeLens.packageResponse.suggestion) return createInvalidCommand(codeLens);

  const { name, version, category, type } = codeLens.packageResponse.suggestion;

  // get the category indicator
  const indicator = indicators[category] + (isWindows ? '' : ' ');
  const indicatedName = indicator
    ? `${indicator}${name}`
    : name;

  // create the indicated command title
  const cmdTitle = type === SuggestionTypes.tag
    ? indicatedName.trim()
    : `${indicatedName} ${version}`.trim();

  // create the suggestion command
  switch (category) {
    case SuggestionCategory.Updateable:
      createUpdateableCommand(cmdTitle, codeLens)
      break;

    case SuggestionCategory.Directory:
      createDirectoryLinkCommand(cmdTitle, codeLens)
      break;

    default:
      createStatusCommand(cmdTitle, codeLens)
      break;
  }
}