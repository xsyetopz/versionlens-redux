import { SuggestionUpdate, VersionUtils } from '#domain/packages';

/**
 * Default function to replace a version in a file, preserving any leading ranges.
 * @param suggestionUpdate The suggestion update information.
 * @param newVersion The new version string to insert.
 * @returns The updated version string.
 */
export function defaultReplaceFn(suggestionUpdate: SuggestionUpdate, newVersion: string): string {
  return VersionUtils.preserveLeadingRange(
    suggestionUpdate.parsedVersion,
    newVersion
  );
}