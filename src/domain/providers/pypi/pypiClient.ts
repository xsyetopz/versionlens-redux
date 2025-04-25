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
import { PypiConfig, PypiHttpClient } from '#domain/providers/pypi';
import { throwUndefinedOrNull } from '@esm-test/guards';
import { coerce } from 'semver';

export class PypiClient implements IPackageClient<null> {

  constructor(
    readonly config: PypiConfig,
    readonly pypiHttpClient: PypiHttpClient,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull("config", config);
    throwUndefinedOrNull("pypiHttpClient", pypiHttpClient);
    throwUndefinedOrNull("logger", logger);
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
    const httpResponse = await this.pypiHttpClient.get(requestPackage.name);

    // process response
    const versionRange = semverSpec.rawVersion;
    const resolved = {
      name: requestPackage.name,
      version: versionRange,
    };

    // extract semver versions only
    const semverVersions = VersionUtils.filterSemverVersions(httpResponse.data)
      .map(x => coerce(x, VersionUtils.loosePrereleases).toString())
      .toSorted(VersionUtils.compareVersionsAndBuilds);

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
      type: semverSpec.type,
      resolved,
      suggestions,
    };
  }
}