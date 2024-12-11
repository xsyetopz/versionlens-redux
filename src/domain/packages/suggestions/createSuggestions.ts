import {
  type PackageSuggestion,
  PackageStatusFactory,
  getPreReleaseSuggestions,
  getReleaseSuggestions,
  getVersionStatus,
  parseVersion
} from '#domain/packages';

export function createSuggestions(
  requestedVersion: string,
  releases: string[],
  prereleases: string[],
  distTagVersion?: string
): Array<PackageSuggestion> {
  if (releases.length === 0 && prereleases.length === 0) {
    // no versions published
    return [PackageStatusFactory.createNoMatchStatus()];
  }

  const parsed = parseVersion(
    requestedVersion,
    releases,
    prereleases,
    distTagVersion
  )

  const status: PackageSuggestion = getVersionStatus(parsed);

  const releaseSuggestions = releases.length > 0
    ? getReleaseSuggestions(requestedVersion, parsed, releases)
    : [];

  const preReleaseSuggestions = prereleases.length > 0
    ? getPreReleaseSuggestions(requestedVersion, prereleases)
    : [];

  return [status, ...releaseSuggestions, ...preReleaseSuggestions];
}