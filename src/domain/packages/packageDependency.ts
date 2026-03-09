import type { PackageManifest } from '#domain/packages';
import {
  type PackageDescriptor,
  type PackageGitDescriptor,
  type PackageGitHubDescriptor,
  type PackageNameDescriptor,
  type PackagePathDescriptor,
  type PackageTextRange,
  type PackageVersionDescriptor,
  PackageDescriptorType
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
    packageRes: PackageManifest,
    readonly descriptors: PackageDescriptor
  ) {
    this.package = packageRes;
    this.descriptors = descriptors;
    this.versionRange = descriptors.getType<PackageVersionDescriptor>(PackageDescriptorType.version)?.versionRange
      ?? descriptors.getType<PackagePathDescriptor>(PackageDescriptorType.path)?.pathRange
      ?? descriptors.getType<PackageGitDescriptor>(PackageDescriptorType.git)?.gitRange
      ?? descriptors.getType<PackageGitHubDescriptor>(PackageDescriptorType.github)?.githubRange
      ?? descriptors.getType<PackageNameDescriptor>(PackageDescriptorType.name)?.nameRange;

    this.nameRange = descriptors.getType<PackageNameDescriptor>(PackageDescriptorType.name)?.nameRange
      ?? this.versionRange;
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
  package: PackageManifest;

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
    return other.versionRange?.start === this.versionRange?.start
      && other.versionRange?.end === this.versionRange?.end
      && other.nameRange?.start === this.nameRange?.start
      && other.nameRange?.end === this.nameRange?.end;
  }

};
