import { ParsedVersion, VersionUtils } from '#domain/packages';
import {
  eq,
  minVersion as getMinVersion,
  maxSatisfying,
  prerelease,
  valid,
  validRange
} from 'semver';

export function parseVersion(
  requestedVersion: string,
  releases: string[],
  prereleases: string[],
  distTagVersion?: string
): ParsedVersion {
  const isFixedVersion = valid(requestedVersion) !== null;
  const isRangeVersion = !isFixedVersion && validRange(requestedVersion) !== null;
  const isPreRelease = isRangeVersion
    ? requestedVersion.includes('-')
    : prerelease(requestedVersion) != null;

  // detect the latest version satisfying the range
  let satisfiesVersion: string = isFixedVersion
    ? VersionUtils.fixedSatisifes(releases, requestedVersion, VersionUtils.loosePrereleases)
    : maxSatisfying(releases, requestedVersion, VersionUtils.loosePrereleases);

  if (!satisfiesVersion && isPreRelease) {
    satisfiesVersion = isFixedVersion
      ? VersionUtils.fixedSatisifes(prereleases, requestedVersion, VersionUtils.loosePrereleases)
      : maxSatisfying(prereleases, requestedVersion, VersionUtils.loosePrereleases);
  }

  let minVersion = null;
  if (isRangeVersion) {
    minVersion = getMinVersion(requestedVersion)?.version;
    satisfiesVersion && satisfiesVersion.startsWith('v') && (minVersion = `v${minVersion}`)
  }

  const latestRelease = distTagVersion || releases[releases.length - 1];
  const latestPreRelease = prereleases[prereleases.length - 1];
  const isLatest = !!latestRelease && !!satisfiesVersion && eq(latestRelease, satisfiesVersion, VersionUtils.loosePrereleases);
  const isLatestPreRelease = isPreRelease && latestPreRelease === satisfiesVersion;
  const hasInvalidRange = isRangeVersion && !minVersion;
  const hasRangeUpdate =
    isRangeVersion &&
    !!satisfiesVersion &&
    satisfiesVersion.startsWith(minVersion) === false;

  return {
    isFixedVersion,
    isRangeVersion,
    isPreRelease,
    isLatest,
    isLatestPreRelease,
    hasInvalidRange,
    hasRangeUpdate,
    minVersion,
    satisfiesVersion,
    latestRelease,
    latestPreRelease: isLatestPreRelease ? latestPreRelease : undefined
  };
}