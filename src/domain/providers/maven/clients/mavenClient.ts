import type { HttpClientResponse, IHttpClient } from '#domain/clients';
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
import { type MavenClientData, MavenConfig, getVersionsFromPackageXml } from '#domain/providers/maven';
import { throwUndefinedOrNull } from '@esm-test/guards';
import { valid } from 'semver';

export class MavenClient implements IPackageClient<MavenClientData> {

  constructor(
    readonly config: MavenConfig,
    readonly httpClient: IHttpClient,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull("config", config);
    throwUndefinedOrNull("httpClient", httpClient);
    throwUndefinedOrNull("logger", logger);
  }

  async fetchPackage(
    request: PackageClientRequest<MavenClientData>
  ): Promise<PackageClientResponse> {
    const requestedPackage = request.parsedDependency.package;
    const semverSpec = VersionUtils.parseSemver(requestedPackage.version);

    const { repositories } = request.clientData;
    const url = repositories[0].url
    let [group, artifact] = requestedPackage.name.split(':');
    let search = group.replace(/\./g, "/") + "/" + artifact
    const queryUrl = `${url}${search}/maven-metadata.xml`;

    try {
      return await this.createRemotePackageDocument(queryUrl, request, semverSpec);
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

  async createRemotePackageDocument(
    url: string,
    request: PackageClientRequest<MavenClientData>,
    semverSpec: SemverSpec
  ): Promise<PackageClientResponse> {
    // fetch package from api
    const httpResponse = await this.httpClient.get(url);

    const { data } = httpResponse;
    const source = PackageSourceType.Registry;
    const versionRange = semverSpec.rawVersion;
    const requestedPackage = request.parsedDependency.package;

    // extract versions form xml
    const rawVersions = getVersionsFromPackageXml(data);

    // extract semver versions only
    const semverVersions = VersionUtils.filterSemverVersions(rawVersions)
      .filter(x => !!valid(x));

    // seperate versions to releases and prereleases
    const { releases, prereleases } = VersionUtils.splitReleasesFromArray(
      semverVersions,
      this.config.prereleaseTagFilter
    );

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
      responseStatus: ClientResponseFactory.mapStatusFromHttpResponse(httpResponse),
      type: semverSpec.type,
      resolved,
      suggestions,
    };
  }

}