import { TSuggestionUpdate, VersionUtils } from 'domain/packages';

export function defaultReplaceFn(suggestionUpdate: TSuggestionUpdate, newVersion: string): string {
  return VersionUtils.preserveLeadingRange(
    suggestionUpdate.parsedVersion,
    newVersion
  );
}