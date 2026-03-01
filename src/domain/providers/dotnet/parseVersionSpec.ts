import { PackageVersionType, VersionUtils } from '#domain/packages';
import type { DotNetVersionSpec, NugetVersionSpec } from '#domain/providers/dotnet';
import { parse, valid, validRange } from 'semver';

/**
 * Parses a NuGet version string into a DotNetVersionSpec.
 * @param rawVersion The raw version string.
 * @returns The parsed version specification.
 */
export function parseVersionSpec(rawVersion: string): DotNetVersionSpec {
  const spec = buildVersionSpec(rawVersion);

  let version: string | undefined;
  let isValidVersion = false;
  let isValidRange = false;

  if (spec && !spec.hasFourSegments) {
    // convert spec to semver
    version = convertVersionSpecToString(spec);
    isValidVersion = valid(version, VersionUtils.loosePrereleases);
    isValidRange = !isValidVersion && validRange(version, VersionUtils.loosePrereleases) !== null;
  }

  const type = isValidVersion
    ? PackageVersionType.Version
    : isValidRange ? PackageVersionType.Range : null

  const resolvedVersion = spec ? version : '';

  return {
    type,
    rawVersion,
    resolvedVersion,
    spec
  };
}

/**
 * Expands short NuGet versions (e.g., '1.0' to '1.0.0').
 * @param value The version string.
 * @returns The expanded version string.
 */
function expandShortVersion(value: string) {
  if (!value ||
    value.indexOf('[') !== -1 ||
    value.indexOf('(') !== -1 ||
    value.indexOf(',') !== -1 ||
    value.indexOf(')') !== -1 ||
    value.indexOf(']') !== -1 ||
    value.indexOf('*') !== -1)
    return value;

  let dotCount = 0;
  for (let i = 0; i < value.length; i++) {
    const c = value[i];
    if (c === '.')
      dotCount++;
    else if (isNaN(parseInt(c)))
      return value;
  }

  let fmtValue = '';
  if (dotCount === 0)
    fmtValue = value + '.0.0';
  else if (dotCount === 1)
    fmtValue = value + '.0';
  else
    return value;

  return fmtValue;
}

/**
 * Builds a NuGet version specification from a string.
 * @param value The version string.
 * @returns The NuGet version specification or null if parsing failed.
 */
function buildVersionSpec(value: string): NugetVersionSpec | null {
  let formattedValue = expandShortVersion(value.trim());
  if (!formattedValue) return null;

  // test if the version is in semver format
  const parsedSemver = parse(formattedValue, { includePrerelease: true });
  if (parsedSemver) {
    return {
      version: formattedValue,
      isMinInclusive: true,
      isMaxInclusive: true,
      hasFourSegments: false,
    };
  }

  try {
    // test if the version is a semver range format
    const parsedNodeRange = validRange(formattedValue, { includePrerelease: true });
    if (parsedNodeRange) {
      return {
        version: parsedNodeRange,
        isMinInclusive: true,
        isMaxInclusive: true,
        hasFourSegments: false,
      };
    }
  } catch { }

  // fail if the string is too short
  if (formattedValue.length < 3) return null;

  const versionSpec: NugetVersionSpec = {};

  // first character must be [ or (
  const first = formattedValue[0];
  if (first === '[')
    versionSpec.isMinInclusive = true;
  else if (first === '(')
    versionSpec.isMinInclusive = false;
  else if (VersionUtils.isFourSegmentedVersion(formattedValue))
    return { hasFourSegments: true }
  else
    return null;

  // last character must be ] or )
  const last = formattedValue[formattedValue.length - 1];
  if (last === ']')
    versionSpec.isMaxInclusive = true;
  else if (last === ')')
    versionSpec.isMaxInclusive = false;

  // remove any [] or ()
  formattedValue = formattedValue.substr(1, formattedValue.length - 2);

  // split by comma
  const parts = formattedValue.split(',');

  // more than 2 is invalid
  if (parts.length > 2)
    return null;
  else if (parts.every(x => !x))
    // must be (,]
    return null;

  // if only one entry then use it for both min and max
  const minVersion = parts[0];
  const maxVersion = (parts.length == 2) ? parts[1] : parts[0];

  // parse the min version
  if (minVersion) {
    const parsedVersion = buildVersionSpec(minVersion);
    if (!parsedVersion) return null;

    versionSpec.minVersionSpec = parsedVersion;
    versionSpec.hasFourSegments = parsedVersion.hasFourSegments;
  }

  // parse the max version
  if (maxVersion) {
    const parsedVersion = buildVersionSpec(maxVersion);
    if (!parsedVersion) return null;

    versionSpec.maxVersionSpec = parsedVersion;
    versionSpec.hasFourSegments = parsedVersion.hasFourSegments;
  }

  return versionSpec;
}

/**
 * Converts a NuGet version specification back into a semver range string.
 * @param versionSpec The NuGet version specification.
 * @returns The range string.
 */
function convertVersionSpecToString(versionSpec: NugetVersionSpec) {
  // x.x.x cases
  if (versionSpec.version
    && versionSpec.isMinInclusive
    && versionSpec.isMaxInclusive)
    return versionSpec.version;

  // [x.x.x] cases
  if (versionSpec.minVersionSpec
    && versionSpec.maxVersionSpec
    && versionSpec.minVersionSpec.version === versionSpec.maxVersionSpec.version
    && versionSpec.isMinInclusive
    && versionSpec.isMaxInclusive)
    return versionSpec.minVersionSpec.version;

  let rangeBuilder = '';

  if (versionSpec.minVersionSpec) {
    rangeBuilder += '>';
    if (versionSpec.isMinInclusive)
      rangeBuilder += '=';
    rangeBuilder += versionSpec.minVersionSpec.version
  }

  if (versionSpec.maxVersionSpec) {
    rangeBuilder += rangeBuilder.length > 0 ? ' ' : '';
    rangeBuilder += '<';
    if (versionSpec.isMaxInclusive)
      rangeBuilder += '=';
    rangeBuilder += versionSpec.maxVersionSpec.version
  }

  return rangeBuilder;
}