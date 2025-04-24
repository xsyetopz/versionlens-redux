import {
  type PackageClientResponse,
  type PackageResponse,
  type PackageSuggestion,
  type PackageClientRequest,
  PackageSourceType,
  PackageVersionType,
} from '#domain/packages';

export function createSuccess<TClientData>(
  providerName: string,
  request: PackageClientRequest<TClientData>,
  response: PackageClientResponse
): Array<PackageResponse> {
  // map the documents to responses
  return response.suggestions.map(
    function (suggestion: PackageSuggestion, order: number): PackageResponse {
      return {
        providerName,
        parsedDependency: request.parsedDependency,
        fetchedPackage: response.resolved,
        packageSource: response.source,
        type: response.type,
        suggestion,
        order,
      };
    }
  );
}

export function createProjectVersionPackageResponse(
  providerName: string,
  request: PackageClientRequest<any>,
  suggestion: PackageSuggestion
) {
  return {
    order: 0,
    providerName,
    suggestion,
    parsedDependency: request.parsedDependency,
    type: PackageVersionType.Version,
    packageSource: PackageSourceType.File,
  }
}