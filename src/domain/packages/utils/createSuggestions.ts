import {
  SuggestionFactory,
  SuggestionStatusText,
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
import { createTaggedPreleaseUpdateable } from '../factories/packageSuggestionFactory';

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
  let status: TPackageSuggestion;

  // determine the current status
  if (!satisfiesVersion) {
    // Cannot find a version that satisfies the range -> suggest only latest
    status = SuggestionFactory.createNoMatchStatus();
  } else if (isLatest) {
    if (hasRangeUpdate) {
      // Theoretically up to date,
      // but it could still be using an older version in the range
      status = SuggestionFactory.createSatisifiesLatestStatus(satisfiesVersion);
    } else {
      // Already up to date -> nothing to do
      status = SuggestionFactory.createMatchesLatestStatus(satisfiesVersion);
    }
  } else if (isFixedVersion) {
    // Not up to date (fixed) -> display the current version
    status = SuggestionFactory.createFixedStatus(satisfiesVersion);
  } else {
    // Not up to date (range) -> display the max satisfying version
    status = SuggestionFactory.createSatisifiesStatus(satisfiesVersion);
  }

  // determine suggestions
  const potentialSuggestions: Array<[SuggestionStatusText, string]> = [];
  const suggestions: Array<TPackageSuggestion> = [];

  // suggest latest?
  const suggestLatest = !isLatest || hasRangeUpdate;
  if (suggestLatest) {
    potentialSuggestions.push([SuggestionStatusText.UpdateLatest, latestVersion]);
  }

  // suggest ranged?
  if (!isLatest && hasRangeUpdate) {
    potentialSuggestions.push([SuggestionStatusText.UpdateRange, satisfiesVersion]);
  }

  // suggest minor and\or patch?
  if (satisfiesVersion || isFixedVersion) {
    const nextMaxMajor = inc(satisfiesVersion ?? versionRange, 'major');
    const nextMaxMinor = inc(satisfiesVersion ?? versionRange, 'minor');
    const nextMaxPatch = inc(satisfiesVersion ?? versionRange, 'patch');

    potentialSuggestions.push(
      [SuggestionStatusText.UpdateMinor, `>=${nextMaxMinor} <${nextMaxMajor}`],
      [SuggestionStatusText.UpdatePatch, `>=${nextMaxPatch} <${nextMaxMinor}`],
    );

    for (const [name, range] of potentialSuggestions) {
      const version = maxSatisfying(releases, range);
      // Only suggest if the version is not already suggested
      if (version && !suggestions.some((s) => s.version === version)) {
        suggestions.push(
          SuggestionFactory.createNextMaxUpdateable(version, name)
        );
      }
    }
  }

  if (!satisfiesVersion && suggestions.length === 0 && suggestLatest) {
    // No satisfying version -> suggest the latest
    suggestions.push(SuggestionFactory.createLatestUpdateable(latestVersion));
  }

  const results = [status, ...suggestions];

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

    results.push(
      createTaggedPreleaseUpdateable(tv.name, tv.version)
    );
  }

  return results;
}