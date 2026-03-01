import {
  type PackageClientResponse,
  type PackageResponse,
  type PackageSuggestion,
  type PackageClientRequest,
  PackageSourceType,
  PackageVersionType,
} from '#domain/packages';

/**
 * Creates an array of PackageResponse objects from a successful fetch response.
 * @template TClientData Type of the client data.
 * @param providerName The name of the provider.
 * @param request The original client request.
 * @param response The response from the fetch operation.
 * @returns An array of package responses, one for each suggestion.
 */
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

/**
 * Creates a PackageResponse for a project version dependency.
 * @param providerName The name of the provider.
 * @param request The original client request.
 * @param suggestion The version suggestion.
 * @returns A package response.
 */
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