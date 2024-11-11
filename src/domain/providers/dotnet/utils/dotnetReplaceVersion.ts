import { TSuggestionUpdate, defaultReplaceFn } from '#domain/packages';
// import { noVersionAttr } from '../parser/dotnetParserTypeFactory';

export function dotnetReplaceVersion(suggestionUpdate: TSuggestionUpdate, newVersion: string): string {
  const insert = suggestionUpdate.parsedVersionPrepend.length > 2;

  return defaultReplaceFn(
    suggestionUpdate,
    // handle cases with blank version attr entries
    insert ?
      `${suggestionUpdate.parsedVersionPrepend}${newVersion}${suggestionUpdate.parsedVersionAppend}` :
      newVersion
  );

}