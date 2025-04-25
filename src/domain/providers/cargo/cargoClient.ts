import type { HttpClientResponse } from '#domain/clients';
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
import {
  type PackageGitDescriptor,
  type PackagePathDescriptor,
  PackageDescriptorType
} from '#domain/parsers';
import type { CargoConfig, CratesClient } from '#domain/providers/cargo';
import { throwUndefinedOrNull } from '@esm-test/guards';

export class CargoClient implements IPackageClient<null> {

  constructor(
    readonly config: CargoConfig,
    readonly cratesClient: CratesClient,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull('config', config);
    throwUndefinedOrNull('cratesClient', cratesClient);
    throwUndefinedOrNull('logger', logger);
  }

  async fetchPackage<TClientData>(
    request: PackageClientRequest<TClientData>
  ): Promise<PackageClientResponse> {
    const requestedPackage = request.parsedDependency.package;

    // return a directory response if this a path type
    const pathDesc = request.parsedDependency.descriptors.getType<PackagePathDescriptor>(
      PackageDescriptorType.path
    );
    if (pathDesc) {
      return await ClientResponseFactory.createDirectory(
        requestedPackage.name,
        requestedPackage.path,
        pathDesc.path
      );
    }

    // return a git response if this a git type
    const gitDesc = request.parsedDependency.descriptors.getType<PackageGitDescriptor>(
      PackageDescriptorType.git
    );
    if (gitDesc) return ClientResponseFactory.createGit();

    // fetch package suggestions from api
    const semverSpec = VersionUtils.parseSemver(requestedPackage.version);
    try {
      return await this.createRemotePackageDocument(request, semverSpec)
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
    request: PackageClientRequest<TClientData>,
    semverSpec: SemverSpec
  ): Promise<PackageClientResponse> {
    // fetch
    const requestPackage = request.parsedDependency.package;
    const jsonResponse = await this.cratesClient.get(requestPackage.name);

    // process response
    const versionRange = semverSpec.rawVersion;
    const resolved = {
      name: requestPackage.name,
      version: versionRange,
    };

    // filter and sort versions
    const rawVersions = jsonResponse.data.versions
      .filter(p => p.yanked === false)
      .map(p => p.num)
      .toSorted(VersionUtils.compareVersionsAndBuilds);

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
      type: semverSpec.type,
      resolved,
      suggestions,
    };
  }
}