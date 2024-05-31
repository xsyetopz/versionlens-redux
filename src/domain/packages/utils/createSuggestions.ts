import {
  SuggestionCategory,
  SuggestionFactory,
  SuggestionStatusText,
  SuggestionTypes,
  TPackageSuggestion,
  VersionUtils,
} from 'domain/packages';
import { Nullable } from 'domain/utils';
import {
  compareLoose,
  inc,
  maxSatisfying,
  minVersion,
  prerelease,
  valid,
  validRange,
} from 'semver';

export function createSuggestions(
  versionRange: string,
  releases: string[],
  prereleases: string[],
  distTagVersion: Nullable<string> = null
): Array<TPackageSuggestion> {
  if (releases.length === 0 && prereleases.length === 0) {
    // No versions available -> nothing to suggest/do
    return [SuggestionFactory.createNoMatchStatus()];
  }

  const isFixedVersion = valid(versionRange) != null;
  const isRangeVersion = !isFixedVersion && validRange(versionRange) != null;
  const isPreRelease = prerelease(versionRange) != null;

  // detect the latest version satisfying the range
  let satisfiesVersion: string = maxSatisfying(
    releases,
    versionRange,
    VersionUtils.loosePrereleases
  );

  if (!satisfiesVersion && isPreRelease) {
    satisfiesVersion = maxSatisfying(
      prereleases,
      versionRange,
      VersionUtils.loosePrereleases
    );
  }

  // get the latest release
  const latestVersion = distTagVersion || releases[releases.length - 1];
  const isLatest = latestVersion === satisfiesVersion;
  const hasRangeUpdate =
    isRangeVersion &&
    satisfiesVersion &&
    satisfiesVersion !== minVersion(versionRange).version;

  const suggestions: Array<TPackageSuggestion> = [];

  // Frist add the actual status

  if (!satisfiesVersion) {
    // Cannot find a version that satisfies the range -> suggest only latest
    suggestions.push(SuggestionFactory.createNoMatchStatus());
  } else if (isLatest) {
    if (hasRangeUpdate) {
      // Theoretically up to date,
      // but it could still be using an older version in the range
      suggestions.push(
        SuggestionFactory.createSatisifiesLatestStatus(satisfiesVersion)
      );
    } else {
      // Already up to date -> nothing to do
      suggestions.push(
        SuggestionFactory.createMatchesLatestStatus(satisfiesVersion)
      );
    }
  } else if (isFixedVersion) {
    // Not up to date (fixed) -> display the current version
    suggestions.push(SuggestionFactory.createFixedStatus(satisfiesVersion));
  } else {
    // Not up to date (range) -> display the max satisfying version
    suggestions.push(
      SuggestionFactory.createSatisifiesStatus(satisfiesVersion)
    );
  }

  // Then suggest available updates

  if (hasRangeUpdate) {
    if (isLatest) {
      suggestions.push(
        SuggestionFactory.createLatestUpdateable(satisfiesVersion)
      );
    } else {
      suggestions.push(
        SuggestionFactory.createRangeUpdateable(satisfiesVersion)
      );
    }
  }

  if (satisfiesVersion) {
    const nextMajor = inc(satisfiesVersion, 'major');
    const nextMinor = inc(satisfiesVersion, 'minor');
    const nextPatch = inc(satisfiesVersion, 'patch');

    const potentialSuggesions: ReadonlyArray<[SuggestionStatusText, string]> = [
      [SuggestionStatusText.Latest, latestVersion],
      [SuggestionStatusText.UpdateLatest, `>=${nextMajor}`],
      [SuggestionStatusText.UpdateMinor, `>=${nextMinor} <${nextMajor}`],
      [SuggestionStatusText.UpdatePatch, `>=${nextPatch} <${nextMinor}`],
    ];

    for (const [name, range] of potentialSuggesions) {
      const version = maxSatisfying(releases, range);
      // Only suggest if the version is not already suggested
      if (version && !suggestions.some((s) => s.version === version)) {
        suggestions.push(
          SuggestionFactory.createLatestUpdateable(version, name)
        );
      }
    }
  } else {
    // No satisfying version -> suggest the latest
    suggestions.push(SuggestionFactory.createLatestUpdateable(latestVersion));
  }

  // roll up prereleases
  const maxSatisfyingPrereleases = VersionUtils.filterPrereleasesGtMinRange(
    versionRange,
    prereleases
  ).sort(compareLoose);

  // group prereleases (latest first)
  const taggedVersions = VersionUtils.extractTaggedVersions(maxSatisfyingPrereleases);
  for (let index = taggedVersions.length - 1; index > -1; index--) {
    const tv = taggedVersions[index];
    if (tv.name === 'latest') break;
    if (tv.version === satisfiesVersion) break;
    if (tv.version === latestVersion) break;
    if (versionRange.includes(tv.version)) break;

    suggestions.push({
      name: tv.name,
      category: SuggestionCategory.Updateable,
      version: tv.version,
      type: SuggestionTypes.prerelease
    });
  }

  return suggestions;
}