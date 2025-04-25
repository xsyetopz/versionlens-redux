import { HttpClientResponse } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import {
  type IPackageClient,
  type PackageClientRequest,
  type PackageClientResponse,
  ClientResponseFactory,
  createSuggestions,
  PackageSourceType,
  PackageStatusFactory,
  PackageVersionType,
  VersionUtils
} from '#domain/packages';
import type { DenoConfig, JsrClient } from '#domain/providers/deno';
import type { NpaSpec, NpmClientData, NpmPackageClient } from '#domain/providers/npm';
import { throwUndefinedOrNull } from '@esm-test/guards';
import npa from 'npm-package-arg';

export class DenoClient implements IPackageClient<NpmClientData> {

  constructor(
    readonly config: DenoConfig,
    readonly jsrClient: JsrClient,
    readonly npmClient: NpmPackageClient,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull("config", config);
    throwUndefinedOrNull("jsrClient", jsrClient);
    throwUndefinedOrNull("npmClient", npmClient);
    throwUndefinedOrNull("logger", logger);
  }

  async fetchPackage(request: PackageClientRequest<NpmClientData>): Promise<PackageClientResponse> {
    const requestedPackage = request.parsedDependency.package;
    const isDenoJsr = requestedPackage.version.startsWith('jsr:');
    const isDenoNpm = requestedPackage.version.startsWith('npm:');
    if (!isDenoJsr && !isDenoNpm) return ClientResponseFactory.createNoSuggestions();
    if (isDenoNpm) return this.npmClient.fetchPackage(request);

    try {
      const npaSpec = npa.resolve(
        requestedPackage.name,
        requestedPackage.version.replaceAll('jsr:', 'npm:'),
        requestedPackage.path
      ) as NpaSpec;

      return this.createRemotePackageDocument(npaSpec);
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

  async createRemotePackageDocument(npaSpec: NpaSpec): Promise<PackageClientResponse> {
    // fetch
    const jsonResponse = await this.jsrClient.get(npaSpec.subSpec.name);

    // process response
    const versionRange = npaSpec.subSpec.rawSpec;
    const resolved = {
      name: npaSpec.subSpec.name,
      version: versionRange,
    };

    // sort versions
    const rawVersions = jsonResponse.data.toSorted(VersionUtils.compareVersionsAndBuilds);

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
      responseStatus: ClientResponseFactory.mapStatusFromJsonResponse(jsonResponse),
      type: PackageVersionType.Alias,
      resolved,
      suggestions,
    };
  }

}