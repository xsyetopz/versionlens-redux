import type { PackageSourceType, PackageVersionType } from '#domain/packages';
import type { PackageTextRange } from '#domain/parsers';

/**
 * Enum representing categories of package suggestions.
 */
export enum SuggestionCategory {
  /** A local directory. */
  Directory = 'Directory',
  /** An error occurred while fetching. */
  Error = 'Error',
  /** The package is up to date with the latest version. */
  Latest = 'Latest',
  /** The package range already satisfies the latest version. */
  SatisfiesLatest = 'SatisfiesLatest',
  /** No matching version found for the range. */
  NoMatch = 'NoMatch',
  /** A version match was found. */
  Match = 'Match',
  /** An update is available. */
  Updateable = 'Updateable',
  /** A build update is available. */
  Build = 'Build'
}

/**
 * Enum representing different version increment types.
 */
export enum SuggestionIncrements {
  major = 'major',
  minor = 'minor',
  patch = 'patch',
  release = 'release',
  prerelease = 'prerelease'
}

/**
 * Enum representing common suggestion status text.
 */
export enum SuggestionStatusText {
  BadRequest = '400 bad request',
  NotAuthorized = '401 not authorized',
  Forbidden = '403 forbidden',
  NotFound = 'not found',
  InternalServerError = '500 internal server error',
  NotSupported = 'not supported',
  ConnectionRefused = 'connection refused',
  ConnectionReset = 'connection reset',
  Invalid = 'invalid entry',
  InvalidVersion = 'invalid version',
  NoMatch = 'no match',
  Satisfies = 'satisfies',
  SatisfiesLatest = 'satisfies latest',
  Latest = 'latest',
  LatestIsPrerelease = 'latest prerelease',
  Fixed = 'fixed',
  UpdateLatest = 'latest',
  UpdateMajor = 'major',
  UpdateMinor = 'minor',
  UpdatePatch = 'patch',
  UpdateLatestPrerelease = 'latest prerelease',
  UpdateRange = 'bump',
  UpdateBuild = 'change build',
}

/**
 * Enum representing the type of suggestion (bitwise).
 */
export enum SuggestionTypes {
  /** A status message. */
  status = 1,
  /** A release version. */
  release = 2,
  /** A prerelease version. */
  prerelease = 4,
  /** A distribution tag. */
  tag = 8
}

/**
 * Represents a single package suggestion.
 */
export type PackageSuggestion = {
  /** The type of suggestion. */
  type: SuggestionTypes,
  /** The category of the suggestion. */
  category: SuggestionCategory,
  /** The display name of the suggestion. */
  name: string,
  /** The suggested version string. */
  version: string,
}

/**
 * Represents the result of parsing a package version.
 */
export type ParsedVersion = {
  /** Whether the version is fixed (no ranges). */
  isFixedVersion: boolean
  /** Whether the version is a range. */
  isRangeVersion: boolean
  /** Whether the version is a prerelease. */
  isPreRelease: boolean
  /** Whether the version is the latest release. */
  isLatest: boolean
  /** Whether the version is the latest prerelease. */
  isLatestPreRelease: boolean
  /** Whether the range is invalid. */
  hasInvalidRange: boolean
  /** Whether there is an update available within the range. */
  hasRangeUpdate: boolean
  /** The minimum version in the range. */
  minVersion: string
  /** The latest version that satisfies the range. */
  satisfiesVersion: string
  /** The absolute latest release version. */
  latestRelease: string
  /** The absolute latest prerelease version if applicable. */
  latestPreRelease?: string
}

/**
 * Represents a group of prerelease versions sharing a common tag.
 */
export type PreReleaseGroup = {
  /** The name of the tag (e.g., 'alpha'). */
  name: string
  /** The order in which it was discovered or published. */
  order: number
  /** The list of versions in this group. */
  versions: string[]
}

/**
 * Information needed to perform a version update in a file.
 */
export type SuggestionUpdate = {
  /** The source of the package. */
  packageSource: PackageSourceType,
  /** The type of version requested. */
  packageVersionType: PackageVersionType | null,

  /** The original package name in the file. */
  parsedName: string,
  /** The original package version in the file. */
  parsedVersion: string,
  /** The range of the version string in the file. */
  parsedVersionRange: PackageTextRange,
  /** Text to prepend to the new version. */
  parsedVersionPrepend: string,
  /** Text to append to the new version. */
  parsedVersionAppend: string,

  /** The name of the package as reported by the registry. */
  fetchedName?: string,
  /** The version of the package currently installed/resolved. */
  fetchedVersion?: string,

  /** The type of the selected suggestion. */
  suggestionType: SuggestionTypes,
  /** The version from the selected suggestion. */
  suggestionVersion: string,
}

/**
 * Function signature for replacing a version string in a file.
 */
export type SuggestionReplaceFunction = (
  suggestionUpdate: SuggestionUpdate,
  version: string
) => string;