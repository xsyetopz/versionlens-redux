import type { HttpClientResponse, IJsonHttpClient } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import {
  type PackageClientResponse,
  ClientResponseFactory,
  PackageSourceType,
  PackageStatusFactory,
  PackageVersionType,
  VersionUtils,
  createSuggestions
} from '#domain/packages';
import { DenoConfig, TJsrApiItem } from '#domain/providers/deno';
import { NpaSpec } from '#domain/providers/npm';
import { throwUndefinedOrNull } from '@esm-test/guards';

export class JsrClient {

  constructor(
    readonly config: DenoConfig,
    readonly jsonClient: IJsonHttpClient,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull("config", config);
    throwUndefinedOrNull("jsonClient", jsonClient);
    throwUndefinedOrNull("logger", logger);
  }

  async fetchPackage(npaSpec: NpaSpec): Promise<PackageClientResponse> {
    const url = `https://jsr.io/${npaSpec.subSpec.name}/meta.json`;

    try {
      return await this.createRemotePackageDocument(url, npaSpec)
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
    npaSpec: NpaSpec
  ): Promise<PackageClientResponse> {
    // fetch package from api
    const httpResponse = await this.jsonClient.get(url);

    // process response
    const versionRange = npaSpec.subSpec.rawSpec;
    const resolved = {
      name: npaSpec.subSpec.name,
      version: versionRange,
    };

    const responseVersions = httpResponse.data as TJsrApiItem;
    let rawVersions = Object.keys(responseVersions.versions)
      .filter(k => !responseVersions.versions[k].yanked)
      .reverse()
      .map(k => k);

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
      responseStatus: ClientResponseFactory.mapStatusFromJsonResponse(httpResponse),
      type: PackageVersionType.Alias,
      resolved,
      suggestions,
    };
  }
}