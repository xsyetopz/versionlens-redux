import {
  PackageDependency,
  PackageSourceType,
  PackageVersionType,
  TPackageNameVersion,
  TPackageSuggestion
} from '#domain/packages';

export type PackageResponse = {
  providerName: string;
  parsedDependency: PackageDependency,
  fetchedPackage?: TPackageNameVersion;
  packageSource?: PackageSourceType;
  type?: PackageVersionType;
  suggestion?: TPackageSuggestion;
  order: number;
};