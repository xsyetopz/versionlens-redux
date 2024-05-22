import { TSuggestionUpdate, defaultReplaceFn } from 'domain/packages';

export function pubReplaceVersion(suggestionUpdate: TSuggestionUpdate, newVersion: string): string {

  return defaultReplaceFn(
    suggestionUpdate,
    // handle cases with blank entries and # comments
    suggestionUpdate.parsedVersion === '#' ?
      `${suggestionUpdate.parsedVersionPrepend}${newVersion}${suggestionUpdate.parsedVersionAppend}` :
      newVersion
  );

}