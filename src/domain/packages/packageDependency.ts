import type { PackageResource } from '#domain/packages';
import type {
  PackageDescriptor,
  PackageNameDescriptor,
  PackagePathDescriptor,
  PackageTextRange,
  PackageVersionDescriptor
} from '#domain/parsers';

/**
 * Represents a parsed package dependency within a file.
 */
export class PackageDependency {

  /**
   * Initializes a new instance of the PackageDependency class.
   * @param packageRes The basic package resource information.
   * @param descriptors The set of descriptors associated with this dependency.
   */
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

  /**
   * The text range of the package name in the source file.
   */
  nameRange: PackageTextRange;

  /**
   * The text range of the package version in the source file.
   */
  versionRange: PackageTextRange;

  /**
   * The package resource information.
   */
  package: PackageResource;

  /**
   * Compares this dependency with another based on name and version.
   * @param other The other dependency to compare.
   * @returns True if the package name and version are equal, otherwise false.
   */
  packageEquals(other: PackageDependency) {
    return other.package.name === this.package.name
      && other.package.version === this.package.version
  }

  /**
   * Compares this dependency with another based on their text ranges in the source file.
   * @param other The other dependency to compare.
   * @returns True if all text ranges are identical, otherwise false.
   */
  rangeEquals(other: PackageDependency) {
    return other.versionRange.start === this.versionRange.start
      && other.versionRange.end === this.versionRange.end
      && other.nameRange.start === this.nameRange.start
      && other.nameRange.end === this.nameRange.end;
  }

};