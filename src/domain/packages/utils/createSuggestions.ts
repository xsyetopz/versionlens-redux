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
  diff,
  inc,
  maxSatisfying,
  minVersion as getMinVersion,
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
  const minVersion = getMinVersion(versionRange).version;
  const hasRangeUpdate =
    isRangeVersion &&
    satisfiesVersion &&
    satisfiesVersion !== minVersion;
  let status: TPackageSuggestion;

  // determine the current status
  if (!satisfiesVersion) {
    // Cannot find a version that satisfies the range -> suggest only latest
    status = PackageStatusFactory.createNoMatchStatus();
  } else if (isLatest) {
    if (hasRangeUpdate) {
      // Theoretically up to date,
      // but it could still be using an older version in the range
      status = PackageStatusFactory.createSatisifiesLatestStatus(satisfiesVersion);
    } else {
      // Already up to date -> nothing to do
      status = PackageStatusFactory.createMatchesLatestStatus(satisfiesVersion);
    }
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

  // suggest ranged?
  if (!isLatest && hasRangeUpdate) {
    const diffNextMaxName = diff(minVersion, satisfiesVersion);
    if (diffNextMaxName === 'minor' || diffNextMaxName === 'patch')
      // prefer the minor or patch text
      potentialSuggestions.push([diffNextMaxName, satisfiesVersion]);
    else
      // special range that isn't the next max minor or patch
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
          UpdateableFactory.createNextMaxUpdateable(version, name)
        );
      }
    }
  }

  if (!satisfiesVersion && suggestions.length === 0 && suggestLatest) {
    // No satisfying version -> suggest the latest
    suggestions.push(UpdateableFactory.createLatestUpdateable(latestVersion));
  }

  // ensure ranged versions are sorted again minor and\or patch versions
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