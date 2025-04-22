import type { TPackageResource } from '#domain/packages';
import type {
  PackageDescriptor,
  TPackageNameDescriptor,
  TPackagePathDescriptor,
  TPackageTextRange,
  TPackageVersionDescriptor
} from '#domain/parsers';

export class PackageDependency {

  constructor(
    packageRes: TPackageResource,
    readonly descriptors: PackageDescriptor
  ) {
    this.package = packageRes;
    this.descriptors = descriptors;
    this.nameRange = descriptors.getType<TPackageNameDescriptor>('name')?.nameRange
    this.versionRange = descriptors.getType<TPackageVersionDescriptor>('version')?.versionRange
      ?? descriptors.getType<TPackagePathDescriptor>('path')?.pathRange
      ?? this.nameRange
  }

  nameRange: TPackageTextRange;

  versionRange: TPackageTextRange;

  package: TPackageResource;

  packageEquals(other: PackageDependency) {
    return other.package.name === this.package.name
      && other.package.version === this.package.version
  }

  rangeEquals(other: PackageDependency) {
    return other.versionRange.start === this.versionRange.start
      && other.versionRange.end === this.versionRange.end
      && other.nameRange.start === this.nameRange.start
      && other.nameRange.end === this.nameRange.end;
  }

};