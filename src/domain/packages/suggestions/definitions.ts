import type { PackageSourceType, PackageVersionType } from '#domain/packages';
import type { TPackageTextRange } from '#domain/parsers';

export enum SuggestionCategory {
  Directory = 'Directory',
  Error = 'Error',
  Latest = 'Latest',
  NoMatch = 'NoMatch',
  Match = 'Match',
  Updateable = 'Updateable',
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
  NotFound = 'package not found',
  InternalServerError = '500 internal server error',
  NotSupported = 'not supported',
  ConnectionRefused = 'connection refused',
  ConnectionReset = 'connection reset',
  Invalid = 'invalid entry',
  InvalidVersion = 'invalid version',
  DirectoryNotFound = 'directory not found',
  NoMatch = 'no match',
  Satisfies = 'satisfies',
  SatisfiesLatest = 'satisfies latest',
  Latest = 'latest',
  LatestIsPrerelease = 'latest prerelease',
  Fixed = 'fixed',
  UpdateLatest = 'latest',
  UpdateMinor = 'minor',
  UpdatePatch = 'patch',
  UpdateLatestPrerelease = 'latest prerelease',
  UpdateRange = 'bump'
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

export type TParsedVersion = {
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
  latestPreRelease: string
}

export type TPreReleaseGroup = {
  name: string
  order: number
  versions: string[]
}

export type TSuggestionUpdate = {
  packageSource: PackageSourceType,
  packageVersionType: PackageVersionType,

  parsedName: string,
  parsedVersion: string,
  parsedVersionRange: TPackageTextRange,
  parsedVersionPrepend: string,
  parsedVersionAppend: string,

  fetchedName?: string,
  fetchedVersion?: string,

  suggestionType: SuggestionTypes,
  suggestionVersion: string,
}

export type TSuggestionReplaceFunction = (
  suggestionUpdate: TSuggestionUpdate,
  version: string
) => string;