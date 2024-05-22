import { PackageDescriptor, TPackageSuggestion, TPackageTextRange } from 'domain/packages';
import { PackageSourceType } from '../clients/ePackageSource';
import { PackageVersionType } from '../definitions/ePackageVersionType';
import { TPackageNameVersion } from '../definitions/tPackageNameVersion';
import { TPackageResource } from '../definitions/tPackageResource';

export type PackageResponse = {
  providerName: string;
  nameRange: TPackageTextRange;
  versionRange: TPackageTextRange;
  parsedPackage: TPackageResource;
  packageDesc: PackageDescriptor;
  fetchedPackage?: TPackageNameVersion;
  packageSource?: PackageSourceType;
  type?: PackageVersionType;
  suggestion?: TPackageSuggestion;
  order: number;
};