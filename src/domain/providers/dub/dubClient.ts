import type { HttpClientResponse, IJsonHttpClient } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import {
  type IPackageClient,
  type TPackageClientRequest,
  type TPackageClientResponse,
  type TPackageSuggestion,
  type TSemverSpec,
  ClientResponseFactory,
  PackageSourceType,
  PackageStatusFactory,
  VersionUtils,
  createSuggestions
} from '#domain/packages';
import { DubConfig } from '#domain/providers/dub';
import { throwUndefinedOrNull } from '@esm-test/guards';
import { valid } from 'semver';

export class DubClient implements IPackageClient<null> {

  constructor(
    readonly config: DubConfig,
    readonly jsonClient: IJsonHttpClient,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull("config", config);
    throwUndefinedOrNull("jsonClient", jsonClient);
    throwUndefinedOrNull("logger", logger);
  }

  async fetchPackage(request: TPackageClientRequest<null>): Promise<TPackageClientResponse> {
    const requestedPackage = request.parsedDependency.package;
    const semverSpec = VersionUtils.parseSemver(requestedPackage.version);
    const url = `${this.config.apiUrl}${encodeURIComponent(requestedPackage.name)}/info`;

    try {
      return await this.createRemotePackageDocument(url, request, semverSpec);
    } catch (error) {
      const errorResponse = error as HttpClientResponse;

      this.logger.debug(
        "Caught exception from %s: %O",
        PackageSourceType.Registry,
        errorResponse
      );

      const suggestion = PackageStatusFactory.createFromHttpStatus(errorResponse.status);
      if (suggestion != null) {
        return ClientResponseFactory.create(
          PackageSourceType.Registry,
          errorResponse,
          [suggestion]
        );
      }

      throw errorResponse;
    }
  }

  async createRemotePackageDocument(
    url: string,
    request: TPackageClientRequest<null>,
    semverSpec: TSemverSpec
  ): Promise<TPackageClientResponse> {
    const requestedPackage = request.parsedDependency.package;
    const query = { minimize: 'true' }

    // fetch package from api
    const httpResponse = await this.jsonClient.get(url, query);

    const packageInfo = httpResponse.data;
    const versionRange = semverSpec.rawVersion;

    const resolved = {
      name: requestedPackage.name,
      version: versionRange,
    };

    const responseStatus = {
      source: httpResponse.source,
      status: httpResponse.status,
    };

    const rawVersions = VersionUtils.extractVersionsFromMap(packageInfo.versions);

    // seperate versions to releases and prereleases
    const { releases, prereleases } = VersionUtils.splitReleasesFromArray(
      rawVersions,
      this.config.prereleaseTagFilter
    );

    // analyse suggestions
    const suggestions = parseSuggestions(
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

export function parseSuggestions(
  versionRange: string,
  releases: string[],
  prereleases: string[]
): Array<TPackageSuggestion> {
  if (releases.length === 0) {
    return [PackageStatusFactory.createNoMatchStatus()]
  }

  const latestRelease = releases[releases.length - 1];
  const isValid = valid(versionRange.replace('~>', ''));

  // checks if this is a repo version
  if (!isValid && versionRange.startsWith('~') && latestRelease === versionRange) {
    return [PackageStatusFactory.createMatchesLatestStatus(versionRange)]
  }

  return createSuggestions(
    versionRange,
    releases,
    prereleases
  );
}