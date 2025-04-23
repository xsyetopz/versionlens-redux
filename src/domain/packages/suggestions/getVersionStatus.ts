import { PackageStatusFactory, ParsedVersion } from '#domain/packages';

export function getVersionStatus(parsed: ParsedVersion) {
  if (parsed.hasInvalidRange) {
    return PackageStatusFactory.createInvalidRangeStatus();
  }

  if (!parsed.satisfiesVersion) {
    // Cannot find a version that satisfies the range -> suggest only latest
    return PackageStatusFactory.createNoMatchStatus();
  }

  if (parsed.isLatest) {
    return parsed.hasRangeUpdate
      // Theoretically up to date,
      // but it could still be using an older version in the range
      ? PackageStatusFactory.createSatisifiesLatestStatus(parsed.satisfiesVersion)
      // Already up to date -> nothing to do
      : PackageStatusFactory.createMatchesLatestStatus(parsed.satisfiesVersion);
  }

  if (parsed.isLatestPreRelease) {
    return PackageStatusFactory.createMatchesLatestStatus(parsed.satisfiesVersion);
  }

  if (parsed.isFixedVersion) {
    // Not up to date (fixed) -> display the current version
    return PackageStatusFactory.createFixedStatus(parsed.satisfiesVersion);
  }

  // Not up to date (range) -> display the max satisfying version
  return PackageStatusFactory.createSatisifiesStatus(parsed.satisfiesVersion);
}