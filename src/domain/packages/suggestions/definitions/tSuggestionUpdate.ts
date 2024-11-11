import { PackageSourceType, PackageVersionType, SuggestionTypes } from '#domain/packages';
import { TPackageTextRange } from '#domain/parsers';

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