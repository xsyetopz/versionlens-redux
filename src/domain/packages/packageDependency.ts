import type { PackageResource } from '#domain/packages';
import type {
  PackageDescriptor,
  PackageNameDescriptor,
  PackagePathDescriptor,
  PackageTextRange,
  PackageVersionDescriptor
} from '#domain/parsers';

export class PackageDependency {

  constructor(
    packageRes: PackageResource,
    readonly descriptors: PackageDescriptor
  ) {
    this.package = packageRes;
    this.descriptors = descriptors;
    this.nameRange = descriptors.getType<PackageNameDescriptor>('name')?.nameRange
    this.versionRange = descriptors.getType<PackageVersionDescriptor>('version')?.versionRange
      ?? descriptors.getType<PackagePathDescriptor>('path')?.pathRange
      ?? this.nameRange
  }

  nameRange: PackageTextRange;

  versionRange: PackageTextRange;

  package: PackageResource;

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