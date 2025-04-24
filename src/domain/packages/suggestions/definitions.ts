import type { PackageSourceType, PackageVersionType } from '#domain/packages';
import type { PackageTextRange } from '#domain/parsers';

export enum SuggestionCategory {
  Directory = 'Directory',
  Error = 'Error',
  Latest = 'Latest',
  NoMatch = 'NoMatch',
  Match = 'Match',
  Updateable = 'Updateable',
  Build = 'Build'
}

export enum SuggestionIncrements {
  major = 'major',
  minor = 'minor',
  patch = 'patch',
  release = 'release',
  prerelease = 'prerelease'
}

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

export enum SuggestionTypes {
  // bitwise
  status = 1,
  release = 2,
  prerelease = 4,
  tag = 8
}

export type PackageSuggestion = {
  type: SuggestionTypes,
  category: SuggestionCategory,
  name: string,
  version: string,
}

export type ParsedVersion = {
  isFixedVersion: boolean
  isRangeVersion: boolean
  isPreRelease: boolean
  isLatest: boolean
  isLatestPreRelease: boolean
  hasInvalidRange: boolean
  hasRangeUpdate: boolean
  minVersion: string
  satisfiesVersion: string
  latestRelease: string
  latestPreRelease?: string
}

export type PreReleaseGroup = {
  name: string
  order: number
  versions: string[]
}

export type SuggestionUpdate = {
  packageSource: PackageSourceType,
  packageVersionType: PackageVersionType,

  parsedName: string,
  parsedVersion: string,
  parsedVersionRange: PackageTextRange,
  parsedVersionPrepend: string,
  parsedVersionAppend: string,

  fetchedName?: string,
  fetchedVersion?: string,

  suggestionType: SuggestionTypes,
  suggestionVersion: string,
}

export type SuggestionReplaceFunction = (
  suggestionUpdate: SuggestionUpdate,
  version: string
) => string;