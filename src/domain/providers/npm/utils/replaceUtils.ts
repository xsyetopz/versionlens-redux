import {
  type SuggestionUpdate,
  PackageSourceType,
  PackageVersionType,
  VersionUtils
} from '#domain/packages';

export function npmReplaceVersion(suggestionUpdate: SuggestionUpdate): string {
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

function replaceGitVersion(suggestionUpdate: SuggestionUpdate): string {
  return suggestionUpdate.parsedVersion.replace(
    suggestionUpdate.fetchedVersion,
    suggestionUpdate.suggestionVersion
  )
}

function replaceAliasVersion(suggestionUpdate: SuggestionUpdate): string {
  // preserve the leading symbol from the existing version
  const preservedLeadingVersion = VersionUtils.preserveLeadingRange(
    suggestionUpdate.fetchedVersion,
    suggestionUpdate.suggestionVersion
  );

  const firstColon = suggestionUpdate.parsedVersion.indexOf(':');
  const registry = suggestionUpdate.parsedVersion.substring(0, firstColon)
  return `${registry}:${suggestionUpdate.fetchedName}@${preservedLeadingVersion}`;
}