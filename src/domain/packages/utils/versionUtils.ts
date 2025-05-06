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

export function lteFromArray(versions: Array<string>, version: string) {
  return versions.filter(
    function (testVersion: string) {
      return lte(testVersion, version);
    }
  );
}

export function removeFourSegmentVersionsFromArray(
  versions: Array<string>
): Array<string> {
  return versions.filter(function (version: string) {
    return isFourSegmentedVersion(version) === false;
  });
}

export function isFixedVersion(versionToCheck: string): boolean {
  const testRange = new Range(versionToCheck, loosePrereleases);
  return valid(versionToCheck) !== null && testRange.set[0][0].operator === "";
}

const isfourSegmentVersionRegex = /^(\d+\.)(\d+\.)(\d+\.)(\*|\d+)$/;
export function isFourSegmentedVersion(versionToCheck: string): boolean {
  return isfourSegmentVersionRegex.test(versionToCheck);
}

export function parseSemver(packageVersion: string): SemverSpec {
  const isVersion = valid(packageVersion, loosePrereleases);
  const isRange = validRange(packageVersion, loosePrereleases);
  return {
    rawVersion: packageVersion,
    type: isVersion !== null
      ? PackageVersionType.Version
      : isRange !== null ? PackageVersionType.Range : null
  };
}

export function filterSemverVersions(versions: Array<string>): Array<string> {
  const semverVersions: Array<string> = [];
  versions.forEach(version => {
    if (validRange(version, loosePrereleases)) semverVersions.push(version);
  });
  return semverVersions;
}

export const extractSymbolFromVersionRegex = /^([^0-9]*)?.*$/;
export const semverLeadingChars = ['^', '~', '<', '<=', '>', '>=', '~>'];
export function preserveLeadingRange(existingVersion: string, newVersion: string) {
  const regExResult = extractSymbolFromVersionRegex.exec(existingVersion);
  const leading = regExResult && regExResult[1];
  if (!leading || !semverLeadingChars.includes(leading))
    return newVersion;

  return `${leading}${newVersion}`;
}

export function fixedSatisifes(versions: string[], fixed: string, options: any): string | null {
  const sf = new SemVer(fixed, options)
  for (let index = versions.length - 1; index >= 0; index--) {
    const v = versions[index];
    const vr = new SemVer(v, options)
    if (sf.compare(vr) === 0 && sf.compareBuild(vr) === 0) return v
  }
  return null
}

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

export function stripBuild(version?: string): string | undefined {
  return version && version.includes('+')
    ? version.substring(0, version.indexOf('+'))
    : version;
}

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

export function compareVersionsAndBuilds(a: string, b: string): 0 | 1 | -1 {
  return compareBuild(a, b, VersionUtils.loosePrereleases)
}