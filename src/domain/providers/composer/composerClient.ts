import type { HttpClientResponse, IJsonHttpClient } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import {
  type IPackageClient,
  type PackageClientRequest,
  type PackageClientResponse,
  type SemverSpec,
  ClientResponseFactory,
  PackageSourceType,
  PackageStatusFactory,
  VersionUtils,
  createSuggestions
} from '#domain/packages';
import { type IPackagistApiItem, ComposerConfig } from '#domain/providers/composer';
import { throwUndefinedOrNull } from '@esm-test/guards';

export class ComposerClient implements IPackageClient<null> {

  constructor(
    readonly config: ComposerConfig,
    readonly jsonClient: IJsonHttpClient,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull("config", config);
    throwUndefinedOrNull("jsonClient", jsonClient);
    throwUndefinedOrNull("logger", logger);
  }

  async fetchPackage<TClientData>(
    request: PackageClientRequest<TClientData>
  ): Promise<PackageClientResponse> {
    const requestedPackage = request.parsedDependency.package;
    const semverSpec = VersionUtils.parseSemver(requestedPackage.version);
    const url = `${this.config.apiUrl}${requestedPackage.name}.json`;

    try {
      return await this.createRemotePackageDocument(url, request, semverSpec)
    } catch (error) {
      const errorResponse = error as HttpClientResponse;

      this.logger.debug(
        "Caught exception from {packageSource}: {error}",
        PackageSourceType.Registry,
        errorResponse
      );

      const suggestion = PackageStatusFactory.createFromHttpStatus(errorResponse.status);
      if (suggestion != null) {
        return ClientResponseFactory.create(
          PackageSourceType.Registry,
          errorResponse,
          [suggestion]
        )
      }

      throw errorResponse;
    }
  }

  async createRemotePackageDocument<TClientData>(
    url: string,
    request: PackageClientRequest<TClientData>,
    semverSpec: SemverSpec
  ): Promise<PackageClientResponse> {
    // fetch package from api
    const httpResponse = await this.jsonClient.get(url);

    const requestPackage = request.parsedDependency.package;
    const versionRange = semverSpec.rawVersion;

    const resolved = {
      name: requestPackage.name,
      version: versionRange,
    };

    const responseStatus = {
      source: httpResponse.source,
      status: httpResponse.status,
    };

    const responseVersions: IPackagistApiItem[] = httpResponse.data.packages[requestPackage.name];

    let rawVersions: string[] = [];
    if (url.indexOf('/p2/') !== -1) {
      rawVersions = responseVersions
        .reverse()
        .map((p: IPackagistApiItem) => p.version);
    } else {
      rawVersions = Object.keys(responseVersions);
    }

    // extract semver versions only
    const semverVersions = VersionUtils.filterSemverVersions(rawVersions);

    // seperate versions to releases and prereleases
    const { releases, prereleases } = VersionUtils.splitReleasesFromArray(
      semverVersions,
      this.config.prereleaseTagFilter
    );

    // analyse suggestions
    const suggestions = createSuggestions(
      versionRange,
      releases,
      prereleases
    );

    return {
      source: PackageSourceType.Registry,
      responseStatus,
      type: semverSpec.type,
      resolved,
      suggestions,
    };
  }
}