import { TSuggestionUpdate, defaultReplaceFn } from '#domain/packages';

export function pubReplaceVersion(suggestionUpdate: TSuggestionUpdate, newVersion: string): string {

  return defaultReplaceFn(
    suggestionUpdate,
    // handle cases for blank entries and # comments
    `${suggestionUpdate.parsedVersionPrepend}${newVersion}${suggestionUpdate.parsedVersionAppend}`
  );

}