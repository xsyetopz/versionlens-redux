import {
  PackageVersionType,
  TPackageNameVersion,
  TPackageVersions,
  TSemverSpec
} from '#domain/packages';
import { Range, lte, prerelease, valid, validRange } from 'semver';

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

export function extractVersionsFromMap(
  versions: Array<TPackageNameVersion>
): Array<string> {
  return versions.map(function (pnv: TPackageNameVersion) {
    return pnv.version;
  });
}

export function extractTaggedVersions(
  versions: Array<string>
): Array<TPackageNameVersion> {
  const results: Array<TPackageNameVersion> = [];
  versions.forEach(function (version) {
    const prereleaseComponents = prerelease(version);
    const isPrerelease = !!prereleaseComponents && prereleaseComponents.length > 0;
    if (isPrerelease === false) return;

    const regexResult = formatTagNameRegex.exec(prereleaseComponents[0]);

    let name = regexResult[0].toLowerCase();
    // capture cases like x.x.x-x.x.x
    if (!name) name = prereleaseComponents.join('.');

    results.push({ name, version })
  });

  return results;
}

export function splitReleasesFromArray(
  versions: Array<string>,
  filterTags: Array<string>
): TPackageVersions {
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

export function parseSemver(packageVersion: string): TSemverSpec {
  const isVersion = valid(packageVersion, loosePrereleases);
  const isRange = validRange(packageVersion, loosePrereleases);
  return {
    rawVersion: packageVersion,
    type: !!isVersion ?
      PackageVersionType.Version :
      !!isRange ? PackageVersionType.Range :
        null,
  };
}

export function filterSemverVersions(versions: Array<string>): Array<string> {
  const semverVersions = [];
  versions.forEach(version => {
    if (validRange(version, loosePrereleases)) semverVersions.push(version);
  });
  return semverVersions;
}

export const extractSymbolFromVersionRegex = /^([^0-9]*)?.*$/;
export const semverLeadingChars = ['^', '~', '<', '<=', '>', '>=', '~>'];
export function preserveLeadingRange(existingVersion, newVersion) {
  const regExResult = extractSymbolFromVersionRegex.exec(existingVersion);
  const leading = regExResult && regExResult[1];
  if (!leading || !semverLeadingChars.includes(leading))
    return newVersion;

  return `${leading}${newVersion}`;
}