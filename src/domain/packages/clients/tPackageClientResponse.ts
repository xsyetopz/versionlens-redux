import {
  PackageSourceType,
  PackageVersionType,
  TPackageClientResponseStatus,
  TPackageNameVersion,
  TPackageSuggestion
} from '#domain/packages';

export type TPackageClientResponse = {

  source: PackageSourceType;

  responseStatus?: TPackageClientResponseStatus;

  type: PackageVersionType;

  resolved?: TPackageNameVersion;

  suggestions: Array<TPackageSuggestion>;

  gitSpec?: any;

};