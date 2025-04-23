import { SuggestionUpdate, VersionUtils } from '#domain/packages';

export function defaultReplaceFn(suggestionUpdate: SuggestionUpdate, newVersion: string): string {
  return VersionUtils.preserveLeadingRange(
    suggestionUpdate.parsedVersion,
    newVersion
  );
}