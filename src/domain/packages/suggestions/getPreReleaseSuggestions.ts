import {
  type PackageSuggestion,
  UpdateableFactory,
  VersionUtils,
  filterPrereleasesGtMinRange
} from '#domain/packages';

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