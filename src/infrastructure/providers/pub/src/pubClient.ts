import { throwUndefinedOrNull } from '@esm-test/guards';
import { HttpClientRequestMethods, HttpClientResponse, IJsonHttpClient } from '#domain/clients';
import { ILogger } from 'domain/logging';
import {
  ClientResponseFactory,
  IPackageClient,
  PackageDescriptorType,
  PackageSourceType,
  PackageStatusFactory,
  TPackageClientRequest,
  TPackageClientResponse,
  TPackageGitDescriptor,
  TPackageHostedDescriptor,
  TPackagePathDescriptor,
  TSemverSpec,
  VersionUtils,
  createSuggestions
} from 'domain/packages';
import semver from 'semver';
import { PubConfig } from './pubConfig';

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

  async fetchPackage(request: TPackageClientRequest<null>): Promise<TPackageClientResponse> {
    const requestedPackage = request.parsedDependency.package;

    // return a directory response if this a path type
    const pathDesc = request.parsedDependency.descriptors.getType<TPackagePathDescriptor>(
      PackageDescriptorType.path
    );
    if (pathDesc) {
      return ClientResponseFactory.createDirectory(
        requestedPackage.name,
        requestedPackage.path,
        pathDesc.path
      );
    }

    // return a git response if this a git type
    const gitDesc = request.parsedDependency.descriptors.getType<TPackageGitDescriptor>(
      PackageDescriptorType.git
    );
    if (gitDesc) {
      return ClientResponseFactory.createGit();
    }

    // parse the version
    const semverSpec = VersionUtils.parseSemver(requestedPackage.version);

    // use the hosted entry if it exists
    const hosted = request.parsedDependency.descriptors.getType<TPackageHostedDescriptor>(
      PackageDescriptorType.hosted
    );

    const url = hosted
      ? `${hosted.hostUrl}/${requestedPackage.name}`
      : `${this.config.apiUrl}/${requestedPackage.name}`;

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
    packageName: string,
    semverSpec: TSemverSpec
  ): Promise<TPackageClientResponse> {

    const query = {};
    const headers = {};

    const httpResponse = await this.jsonClient.request(
      HttpClientRequestMethods.get,
      url,
      query,
      headers
    );

    const packageInfo = httpResponse.data;

    const versionRange = semverSpec.rawVersion;

    const resolved = {
      name: packageName,
      version: versionRange,
    };

    const responseStatus = {
      source: httpResponse.source,
      status: httpResponse.status,
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
      responseStatus,
      type: semverSpec.type,
      resolved,
      suggestions,
    };
  }

}