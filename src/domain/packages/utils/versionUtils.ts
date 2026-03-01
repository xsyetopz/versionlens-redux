import {
  type PackageNameVersion,
  type PackageVersions,
  type SemverSpec,
  PackageVersionType,
  VersionUtils
} from '#domain/packages';
import {
  coerce,
  compareBuild,
  eq,
  lte,
  major,
  prerelease,
  Range,
  SemVer,
  valid,
  validRange
} from 'semver';

export const formatTagNameRegex = /^[^0-9\-]*/;
export const loosePrereleases = { loose: true, includePrerelease: true };

/**
 * Filters a list of prerelease versions based on allowed tags.
 * @param prereleases The list of prerelease versions.
 * @param filterTags The list of allowed tags (e.g., ['beta', 'rc']).
 * @returns An array of filtered prerelease versions.
 */
export function filterPrereleaseTags(
  prereleases: Array<string>,
  filterTags: Array<string>
): Array<string> {
  if (filterTags.length === 0) return prereleases;

  return prereleases.filter(
    (version: string) => {
      const parts = prerelease(version);
      return filterTags.some(x => x.toLowerCase() === parts[0]);
    }
  );
}

/**
 * Extracts distribution tag names from a list of versions.
 * @param versions The list of versions.
 * @returns An array of package name and version pairs for each tag.
 */
export function extractTaggedVersions(versions: Array<string>): Array<PackageNameVersion> {
  const results: Array<PackageNameVersion> = [];
  versions.forEach(function (version) {
    const prereleaseComponents = prerelease(version);
    const isPrerelease = !!prereleaseComponents && prereleaseComponents.length > 0;
    if (isPrerelease === false) return;

    const regexResult = formatTagNameRegex.exec(prereleaseComponents[0]);
    if (regexResult === null) return;

    let name = regexResult[0].toLowerCase();
    // capture cases like x.x.x-x.x.x
    if (!name) name = prereleaseComponents.join('.');

    results.push({ name, version });
  });

  return results;
}

/**
 * Splits an array of versions into release and prerelease groups.
 * @param versions The list of versions.
 * @param filterTags The list of allowed prerelease tags.
 * @returns A PackageVersions object.
 */
export function splitReleasesFromArray(
  versions: Array<string>,
  filterTags: Array<string>
): PackageVersions {
  const releases: Array<string> = [];
  const prereleases: Array<string> = [];

  versions.forEach(function (version: string) {
    if (prerelease(version))
      prereleases.push(version);
    else
      releases.push(version);
  });

  const filteredPrereleases = filterPrereleaseTags(prereleases, filterTags);

  return { releases, prereleases: filteredPrereleases };
}

/**
 * Filters versions that are less than or equal to a specific version.
 * @param versions The list of versions.
 * @param version The version to compare against.
 * @returns An array of versions.
 */
export function lteFromArray(versions: Array<string>, version: string) {
  return versions.filter(
    function (testVersion: string) {
      return lte(testVersion, version);
    }
  );
}

/**
 * Removes versions that have four segments (e.g., 1.2.3.4) from an array.
 * @param versions The list of versions.
 * @returns An array of versions.
 */
export function removeFourSegmentVersionsFromArray(
  versions: Array<string>
): Array<string> {
  return versions.filter(function (version: string) {
    return isFourSegmentedVersion(version) === false;
  });
}

/**
 * Checks if a version string represents a fixed version (no ranges).
 * @param versionToCheck The version string to check.
 * @returns True if the version is fixed, otherwise false.
 */
export function isFixedVersion(versionToCheck: string): boolean {
  const testRange = new Range(versionToCheck, loosePrereleases);
  return valid(versionToCheck) !== null && testRange.set[0][0].operator === "";
}

const isfourSegmentVersionRegex = /^(\d+\.)(\d+\.)(\d+\.)(\*|\d+)$/;
/**
 * Checks if a version string has four segments.
 * @param versionToCheck The version string to check.
 * @returns True if the version has four segments, otherwise false.
 */
export function isFourSegmentedVersion(versionToCheck: string): boolean {
  return isfourSegmentVersionRegex.test(versionToCheck);
}

/**
 * Parses a version string into a SemverSpec.
 * @param packageVersion The version string to parse.
 * @returns A SemverSpec object or null if parsing failed.
 */
export function parseSemver(packageVersion: string): SemverSpec | null {
  const isVersion = valid(packageVersion, loosePrereleases);
  const isRange = validRange(packageVersion, loosePrereleases);
  const type = isVersion !== null
    ? PackageVersionType.Version
    : isRange !== null ? PackageVersionType.Range : null;
  if (type === null) return null;
  return {
    rawVersion: packageVersion,
    type
  };
}

/**
 * Filters a list of versions to only those that are valid semver ranges.
 * @param versions The list of versions.
 * @returns An array of valid semver versions.
 */
export function filterSemverVersions(versions: Array<string>): Array<string> {
  const semverVersions: Array<string> = [];
  versions.forEach(version => {
    if (validRange(version, loosePrereleases)) semverVersions.push(version);
  });
  return semverVersions;
}

export const extractSymbolFromVersionRegex = /^([^0-9]*)?.*$/;
export const semverLeadingChars = ['^', '~', '<', '<=', '>', '>=', '~>'];
/**
 * Preserves the leading range symbol (e.g., ^, ~) when updating a version.
 * @param existingVersion The current version string in the file.
 * @param newVersion The new version string to use.
 * @returns The new version string with the original range symbol prepended.
 */
export function preserveLeadingRange(existingVersion: string, newVersion: string) {
  const regExResult = extractSymbolFromVersionRegex.exec(existingVersion);
  const leading = regExResult && regExResult[1];
  if (!leading || !semverLeadingChars.includes(leading))
    return newVersion;

  return `${leading}${newVersion}`;
}

/**
 * Finds a version in a list that exactly matches a fixed version.
 * @param versions The list of versions to search.
 * @param fixed The fixed version to match.
 * @param options Semver options.
 * @returns The matching version string or null.
 */
export function fixedSatisifes(versions: string[], fixed: string, options: any): string | null {
  const sf = new SemVer(fixed, options)
  for (let index = versions.length - 1; index >= 0; index--) {
    const v = versions[index];
    const vr = new SemVer(v, options)
    if (sf.compare(vr) === 0 && sf.compareBuild(vr) === 0) return v
  }
  return null
}

/**
 * Finds all build variations for a specific version in a list.
 * @param fixed The fixed version.
 * @param versions The list of versions to search.
 * @param options Semver options.
 * @returns An array of version strings with different build metadata.
 */
export function findVersionBuilds(fixed: string, versions: string[], options: any): string[] {
  const results: string[] = [];
  for (let index = 0; index < versions.length; index++) {
    const version = versions[index];
    if (results.includes(version)) continue;

    const coerced = coerce(version, options);
    if (coerced === null) continue;

    eq(fixed, coerced, options) && results.push(version);
  }

  return results;
}

/**
 * Strips build metadata from a version string.
 * @param version The version string.
 * @returns The version string without build metadata.
 */
export function stripBuild(version?: string): string | undefined {
  return version && version.includes('+')
    ? version.substring(0, version.indexOf('+'))
    : version;
}

/**
 * Finds the next major version in a list starting from a specific version.
 * @param startFromVersion The version to start from.
 * @param versions The list of versions.
 * @returns The next major version string or null.
 */
export function findNextMajor(startFromVersion: string, versions: string[]): string | null {
  let index = versions.indexOf(startFromVersion);
  if (index === -1) return null;
  try {
    const startMajor = major(startFromVersion);
    for (; index < versions.length; index++) {
      const testMajor = major(versions[index], true)
      if (testMajor > startMajor) return testMajor
    }
  } catch { }
  return null
}

/**
 * Compares two version strings including build metadata.
 * @param a The first version.
 * @param b The second version.
 * @returns A comparison result (0, 1, or -1).
 */
export function compareVersionsAndBuilds(a: string, b: string): 0 | 1 | -1 {
  return compareBuild(a, b, VersionUtils.loosePrereleases)
}