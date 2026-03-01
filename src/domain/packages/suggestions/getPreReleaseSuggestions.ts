import {
  type PackageSuggestion,
  UpdateableFactory,
  VersionUtils,
  filterPrereleasesGtMinRange
} from '#domain/packages';

/**
 * Generates suggestions for prerelease versions.
 * @param fixedOrRangedVersion The current version range in the file.
 * @param prereleases The list of available prerelease versions.
 * @returns An array of prerelease suggestions.
 */
export function getPreReleaseSuggestions(
  fixedOrRangedVersion: string,
  prereleases: string[]
): PackageSuggestion[] {
  const maxSatisfyingPrereleases = filterPrereleasesGtMinRange(
    fixedOrRangedVersion,
    prereleases
  );

  if (maxSatisfyingPrereleases.length === 0) return [];

  // get unique tag names
  const taggedVersions = VersionUtils.extractTaggedVersions(maxSatisfyingPrereleases);

  // map name to tag-name
  const suggestions = maxSatisfyingPrereleases.map(
    (x, i) => UpdateableFactory.createTaggedPreleaseUpdateable(taggedVersions[i].name, x)
  );

  // order releases  (latest first)
  return suggestions.toReversed();
}