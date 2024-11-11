import { TSuggestionUpdate, defaultReplaceFn } from '#domain/packages';

export function goReplaceVersion(suggestionUpdate: TSuggestionUpdate, newVersion: string): string {
  const insert = suggestionUpdate.parsedVersionPrepend.length > 0;

  return defaultReplaceFn(
    suggestionUpdate,
    // handle cases with blank version attr entries
    insert
      ? `${suggestionUpdate.parsedVersionPrepend}${newVersion}${suggestionUpdate.parsedVersionAppend}`
      : newVersion
  );

}