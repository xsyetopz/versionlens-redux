import { PackageDescriptorType, PackageResponse, TPackageVersionDescriptor, TSuggestionUpdate } from 'domain/packages';

export function mapToSuggestionUpdate(packageResponse: PackageResponse): TSuggestionUpdate {
  let parsedVersionPrepend = "";
  let parsedVersionAppend = "";

  if (packageResponse.packageDesc.hasType(PackageDescriptorType.version)) {
    const versionDesc = packageResponse.packageDesc.getType<TPackageVersionDescriptor>(PackageDescriptorType.version);
    parsedVersionPrepend = versionDesc.versionPrepend;
    parsedVersionAppend = versionDesc.versionAppend;
  }

  return {
    packageSource: packageResponse.packageSource,
    packageVersionType: packageResponse.type,

    parsedName: packageResponse.parsedPackage.name,
    parsedVersion: packageResponse.parsedPackage.version,
    parsedVersionRange: packageResponse.versionRange,
    parsedVersionPrepend,
    parsedVersionAppend,

    fetchedName: packageResponse.fetchedPackage?.name,
    fetchedVersion: packageResponse.fetchedPackage?.version,

    suggestionType: packageResponse.suggestion.type,
    suggestionVersion: packageResponse.suggestion.version,
  }
}