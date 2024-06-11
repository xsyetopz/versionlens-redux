import {
  PackageResponse,
  PackageSourceType,
  PackageVersionType,
  TPackageClientRequest,
  TPackageClientResponse,
  TPackageSuggestion
} from '#domain/packages';

export function createSuccess<TClientData>(
  providerName: string,
  request: TPackageClientRequest<TClientData>,
  response: TPackageClientResponse
): Array<PackageResponse> {
  // map the documents to responses
  return response.suggestions.map(
    function (suggestion: TPackageSuggestion, order: number): PackageResponse {
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
  request: TPackageClientRequest<any>,
  suggestion: TPackageSuggestion
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