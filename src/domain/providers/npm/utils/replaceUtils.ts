import {
  PackageSourceType,
  PackageVersionType,
  TSuggestionUpdate,
  VersionUtils
} from '#domain/packages';

export function npmReplaceVersion(suggestionUpdate: TSuggestionUpdate): string {
  if (suggestionUpdate.packageSource === PackageSourceType.Github) {
    return replaceGitVersion(suggestionUpdate);
  }

  if (suggestionUpdate.packageVersionType === PackageVersionType.Alias) {
    return replaceAliasVersion(suggestionUpdate);
  }

  // fallback to default
  return VersionUtils.preserveLeadingRange(
    suggestionUpdate.parsedVersion,
    suggestionUpdate.suggestionVersion
  );
}

function replaceGitVersion(suggestionUpdate: TSuggestionUpdate): string {
  return suggestionUpdate.parsedVersion.replace(
    suggestionUpdate.fetchedVersion,
    suggestionUpdate.suggestionVersion
  )
}

function replaceAliasVersion(suggestionUpdate: TSuggestionUpdate): string {
  // preserve the leading symbol from the existing version
  const preservedLeadingVersion = VersionUtils.preserveLeadingRange(
    suggestionUpdate.fetchedVersion,
    suggestionUpdate.suggestionVersion
  );

  return `npm:${suggestionUpdate.fetchedName}@${preservedLeadingVersion}`;
}