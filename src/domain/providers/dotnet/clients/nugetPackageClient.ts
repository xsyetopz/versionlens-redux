import type { HttpClientResponse, IJsonHttpClient } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import {
  type IPackageClient,
  type TPackageClientRequest,
  type TPackageClientResponse,
  ClientResponseFactory,
  PackageSourceType,
  PackageStatusFactory,
  PackageVersionType,
  VersionUtils,
  createSuggestions
} from '#domain/packages';
import { type NuGetClientData, DotNetConfig, parseVersionSpec } from '#domain/providers/dotnet';
import { ensureEndSlash } from '#domain/utils';
import { throwUndefinedOrNull } from '@esm-test/guards';

export class NuGetPackageClient implements IPackageClient<NuGetClientData> {

  constructor(
    readonly config: DotNetConfig,
    readonly jsonClient: IJsonHttpClient,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull("config", config);
    throwUndefinedOrNull("jsonClient", jsonClient);
    throwUndefinedOrNull("logger", logger);
  }

  async fetchPackage(
    request: TPackageClientRequest<NuGetClientData>
  ): Promise<TPackageClientResponse> {
    const urls = request.clientData.serviceUrls;
    const resourceUrl = urls[request.attempt];

    try {
      return await this.fetch(resourceUrl, request);
    }
    catch (error) {
      const errorResponse = error as HttpClientResponse;

      this.logger.debug(
        "request failed for '{packageName}' from '{resourceUrl}': {error}",
        request.parsedDependency.package.name,
        resourceUrl,
        errorResponse
      );

      // increase the attempt number
      request.attempt++;

      // only retry if 404 and we have more urls to try
      if (errorResponse.status === 404 && request.attempt < urls.length) {
        this.logger.debug(
          "attempting to fetch '{packageName}' from '{url}'",
          request.parsedDependency.package.name,
          new URL(urls[request.attempt])
        );
        return this.fetchPackage(request);
      }

      // attempt to create a suggestion from the http status
      const suggestion = PackageStatusFactory.createFromHttpStatus(errorResponse.status);
      if (suggestion != null) {
        return ClientResponseFactory.create(
          PackageSourceType.Registry,
          errorResponse,
          [suggestion]
        );
      }

      // unexpected
      return Promise.reject(errorResponse);
    };
  }

  async fetch(
    resourceUrl: string,
    request: TPackageClientRequest<NuGetClientData>
  ): Promise<TPackageClientResponse> {
    const requestedPackage = request.parsedDependency.package;
    const packageUrl = ensureEndSlash(resourceUrl)
      + `${requestedPackage.name.toLowerCase()}/index.json`;

    const jsonResponse = await this.jsonClient.get(packageUrl);

    const { data } = jsonResponse;

    const source = PackageSourceType.Registry;

    const packageInfo = data;

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
    const rawVersions = VersionUtils.filterSemverVersions(packageInfo.versions);

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
        releases.length > 0 ? releases[releases.length - 1] : null,
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