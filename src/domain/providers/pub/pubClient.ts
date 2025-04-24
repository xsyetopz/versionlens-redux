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
import {
  type PackageGitDescriptor,
  type PackageHostedDescriptor,
  type PackagePathDescriptor,
  PackageDescriptorType
} from '#domain/parsers';
import { PubConfig } from '#domain/providers/pub';
import { throwUndefinedOrNull } from '@esm-test/guards';
import semver from 'semver';

export class PubClient implements IPackageClient<null> {

  constructor(
    readonly config: PubConfig,
    readonly jsonClient: IJsonHttpClient,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull("config", config);
    throwUndefinedOrNull("jsonClient", jsonClient);
    throwUndefinedOrNull("logger", logger);
  }

  async fetchPackage(request: PackageClientRequest<null>): Promise<PackageClientResponse> {
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
    if (gitDesc) {
      return ClientResponseFactory.createGit();
    }

    // parse the version
    const semverSpec = VersionUtils.parseSemver(requestedPackage.version);

    // use the hosted entry if it exists
    const hosted = request.parsedDependency.descriptors.getType<PackageHostedDescriptor>(
      PackageDescriptorType.hosted
    );

    const url = hosted
      ? `${hosted.hostUrl}/${requestedPackage.name}`
      : `${this.config.apiUrl}${requestedPackage.name}`;

    try {
      return await this.createRemotePackageDocument(
        url,
        requestedPackage.name,
        semverSpec
      );
    }
    catch (error) {
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
        );
      }

      throw errorResponse;
    }
  }

  async createRemotePackageDocument(
    url: string,
    packageName: string,
    semverSpec: SemverSpec
  ): Promise<PackageClientResponse> {
    // fetch package from api
    const jsonResponse = await this.jsonClient.get(url);

    const packageInfo = jsonResponse.data;

    const versionRange = semverSpec.rawVersion;

    const resolved = {
      name: packageName,
      version: versionRange,
    };

    // remove redacted versions
    const liveVersions = packageInfo.versions.filter(pkg => !pkg.retracted);

    // map each package.version in to an array
    const rawVersions = VersionUtils.extractVersionsFromMap(liveVersions);

    // seperate versions to releases and prereleases
    const { releases, prereleases } = VersionUtils.splitReleasesFromArray(
      rawVersions.sort(semver.compareLoose),
      this.config.prereleaseTagFilter
    );

    // analyse suggestions
    const suggestions = createSuggestions(
      versionRange,
      releases,
      prereleases
    );

    // return PackageDocument
    return {
      source: PackageSourceType.Registry,
      responseStatus: ClientResponseFactory.mapStatusFromJsonResponse(jsonResponse),
      type: semverSpec.type,
      resolved,
      suggestions,
    };
  }

}