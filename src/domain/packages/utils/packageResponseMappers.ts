import type { PackageResponse, SuggestionUpdate } from '#domain/packages';
import { type PackageVersionDescriptor, PackageDescriptorType } from '#domain/parsers';

/**
 * Maps a PackageResponse to a SuggestionUpdate object.
 * @param packageResponse The package response to map.
 * @returns A SuggestionUpdate object.
 */
export function mapToSuggestionUpdate(packageResponse: PackageResponse): SuggestionUpdate {
  let parsedVersionPrepend = "";
  let parsedVersionAppend = "";

  if (!packageResponse.suggestion) throw new Error("packageResponse.suggestion is required");

  if (packageResponse.parsedDependency.descriptors.hasType(PackageDescriptorType.version)) {
    const versionDesc = packageResponse.parsedDependency.descriptors.getType<PackageVersionDescriptor>(PackageDescriptorType.version);
    parsedVersionPrepend = versionDesc.versionPrepend;
    parsedVersionAppend = versionDesc.versionAppend;
  }

  return {
    packageSource: packageResponse.packageSource,
    packageVersionType: packageResponse.type,

    parsedName: packageResponse.parsedDependency.package.name,
    parsedVersion: packageResponse.parsedDependency.package.version,
    parsedVersionRange: packageResponse.parsedDependency.versionRange,
    parsedVersionPrepend,
    parsedVersionAppend,

    fetchedName: packageResponse.fetchedPackage?.name,
    fetchedVersion: packageResponse.fetchedPackage?.version,

    suggestionType: packageResponse.suggestion.type,
    suggestionVersion: packageResponse.suggestion.version,
  }
}