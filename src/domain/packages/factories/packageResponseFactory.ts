import { TPackageClientRequest } from "../clients/tPackageClientRequest";
import { TPackageClientResponse } from "../clients/tPackageClientResponse";
import { PackageResponse } from "../definitions/tPackageResponse";
import { TPackageSuggestion } from "../suggestions/tPackageSuggestion";

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
        nameRange: request.dependency.nameRange,
        versionRange: request.dependency.versionRange,
        parsedPackage: request.dependency.package,
        packageDesc: request.dependency.packageDesc,
        fetchedPackage: response.resolved,
        packageSource: response.source,
        type: response.type,
        suggestion,
        order,
      };
    }
  );
}