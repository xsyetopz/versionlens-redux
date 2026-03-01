import {
  type PackageSuggestion,
  PackageStatusFactory,
  getPreReleaseSuggestions,
  getReleaseSuggestions,
  getVersionStatus,
  parseVersion
} from '#domain/packages';

/**
 * Creates an array of package suggestions based on the requested version and available versions.
 * @param requestedVersion The version string requested in the package file.
 * @param releases The list of available release versions.
 * @param prereleases The list of available prerelease versions.
 * @param distTagVersion Optional version associated with a distribution tag (e.g., 'latest').
 * @returns An array of package suggestions.
 */
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