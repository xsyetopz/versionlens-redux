import {
  PackageStatusFactory,
  SuggestionStatusText,
  TPackageSuggestion,
  UpdateableFactory,
  VersionUtils,
} from 'domain/packages';
import { Nullable } from 'domain/utils';
import {
  compare,
  compareLoose,
  minVersion as getMinVersion,
  inc,
  maxSatisfying,
  prerelease,
  valid,
  validRange
} from 'semver';

export function createSuggestions(
  versionRange: string,
  releases: string[],
  prereleases: string[],
  distTagVersion: Nullable<string> = null
): Array<TPackageSuggestion> {
  if (releases.length === 0 && prereleases.length === 0) {
    // No versions available -> nothing to suggest/do
    return [PackageStatusFactory.createNoMatchStatus()];
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

  let minVersion = null;
  if (isRangeVersion) minVersion = getMinVersion(versionRange)?.version;

  const hasRangeUpdate =
    isRangeVersion &&
    satisfiesVersion &&
    satisfiesVersion !== minVersion;

  let status: TPackageSuggestion;

  // determine the current status
  if (isRangeVersion && !minVersion) {
    // has a invalid range
    status = PackageStatusFactory.createInvalidRangeStatus();
  }
  else if (!satisfiesVersion) {
    // Cannot find a version that satisfies the range -> suggest only latest
    status = PackageStatusFactory.createNoMatchStatus();
  } else if (isLatest) {
    status = hasRangeUpdate
      // Theoretically up to date,
      // but it could still be using an older version in the range
      ? PackageStatusFactory.createSatisifiesLatestStatus(satisfiesVersion)
      // Already up to date -> nothing to do
      : PackageStatusFactory.createMatchesLatestStatus(satisfiesVersion);
  } else if (isFixedVersion) {
    // Not up to date (fixed) -> display the current version
    status = PackageStatusFactory.createFixedStatus(satisfiesVersion);
  } else {
    // Not up to date (range) -> display the max satisfying version
    status = PackageStatusFactory.createSatisifiesStatus(satisfiesVersion);
  }

  // determine suggestions
  const potentialSuggestions: Array<[SuggestionStatusText, string]> = [];
  const suggestions: Array<TPackageSuggestion> = [];

  // suggest latest?
  const suggestLatest = !isLatest || hasRangeUpdate;
  if (suggestLatest) {
    potentialSuggestions.push([SuggestionStatusText.UpdateLatest, latestVersion]);
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
  }

  // suggest ranged?
  if (!isLatest && hasRangeUpdate) {
    potentialSuggestions.push([SuggestionStatusText.UpdateRange, satisfiesVersion]);
  }

  // reduce the potential suggestions
  for (const [name, range] of potentialSuggestions) {
    const version = maxSatisfying(releases, range);
    // Only suggest if the version is not already suggested
    if (version && !suggestions.some((s) => s.version === version)) {
      suggestions.push(
        UpdateableFactory.createNextMaxUpdateable(version, name)
      );
    }
  }

  if (!satisfiesVersion && suggestions.length === 0 && suggestLatest) {
    // No satisfying version -> suggest the latest
    suggestions.push(UpdateableFactory.createLatestUpdateable(latestVersion));
  }

  // sort the suggested versions (latest first)
  const orderedSuggestions = suggestions.sort((a, b) => compare(b.version, a.version));

  const results = [status, ...orderedSuggestions];

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
      UpdateableFactory.createTaggedPreleaseUpdateable(tv.name, tv.version)
    );
  }

  return results;
}