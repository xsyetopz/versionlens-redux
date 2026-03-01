import type { ILogger } from '#domain/logging';
import {
  type PackageClientRequest,
  type PackageClientResponse,
  ClientResponseFactory,
  PackageSourceType,
  PackageVersionType,
  VersionUtils,
  createSuggestions
} from '#domain/packages';
import {
  type DotNetConfig,
  type NuGetClient,
  type NuGetClientData,
  parseVersionSpec
} from '#domain/providers/dotnet';
import { throwUndefinedOrNull } from '@esm-test/guards';

/**
 * Resolves package suggestions for DotNet dependencies from the NuGet registry.
 */
export class DotnetSuggestionResolver {

  /**
   * Initializes a new instance of the DotnetSuggestionResolver class.
   * @param config The configuration for the DotNet provider.
   * @param nugetClient The client used to interact with NuGet.
   * @param logger The logger for this resolver.
   */
  constructor(
    readonly config: DotNetConfig,
    readonly nugetClient: NuGetClient,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull("config", config);
    throwUndefinedOrNull("nugetClient", nugetClient);
    throwUndefinedOrNull("logger", logger);
  }

  /**
   * Resolves suggestions from the NuGet registry.
   * @param request The package client request.
   * @returns A promise resolving to the package client response.
   */
  async fromNuGet(request: PackageClientRequest<NuGetClientData>): Promise<PackageClientResponse> {
    // fetch
    const requestedPackage = request.parsedDependency.package;
    const jsonResponse = await this.nugetClient.get(
      requestedPackage.name,
      request.clientData.serviceUrls
    );

    // process response
    const { data } = jsonResponse;

    const source = PackageSourceType.Registry;

    // parse nuget range expressions
    const dotnetSpec = parseVersionSpec(requestedPackage.version);

    // four segment is not supported
    if (dotnetSpec.spec && dotnetSpec.spec.hasFourSegments) {
      return ClientResponseFactory.create(
        PackageSourceType.Registry,
        jsonResponse,
        [],
      );
    }

    // sanitize to semver only versions
    const rawVersions = VersionUtils.filterSemverVersions(data.versions)
      .toSorted(VersionUtils.compareVersionsAndBuilds);

    // seperate versions to releases and prereleases
    const { releases, prereleases } = VersionUtils.splitReleasesFromArray(
      rawVersions,
      this.config.prereleaseTagFilter
    );

    // return no match if null type
    if (dotnetSpec.type === null) {
      return ClientResponseFactory.createNoMatch(
        source,
        PackageVersionType.Version,
        ClientResponseFactory.createResponseStatus(jsonResponse.source, 404),
        // suggest the latest release if available
        releases.length > 0 ? releases[releases.length - 1] : undefined,
      );
    }

    const versionRange = dotnetSpec.resolvedVersion;

    const resolved = {
      name: requestedPackage.name,
      version: versionRange,
    };

    // analyse suggestions
    const suggestions = createSuggestions(
      versionRange,
      releases,
      prereleases
    );

    return {
      source,
      responseStatus: ClientResponseFactory.mapStatusFromJsonResponse(jsonResponse),
      type: dotnetSpec.type,
      resolved,
      suggestions,
    };
  }

}