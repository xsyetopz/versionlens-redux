import { PackageDescriptor, TPackageResource, TPackageTextRange } from '#domain/packages';

export class PackageDependency {

  constructor(
    packageRes: TPackageResource,
    nameRange: TPackageTextRange,
    versionRange: TPackageTextRange,
    descriptors: PackageDescriptor
  ) {
    this.package = packageRes;
    this.nameRange = nameRange;
    this.versionRange = versionRange;
    this.descriptors = descriptors;
  }

  nameRange: TPackageTextRange;

  versionRange: TPackageTextRange;

  package: TPackageResource;

  descriptors: PackageDescriptor;

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