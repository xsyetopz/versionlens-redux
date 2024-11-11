import {
  HttpClientRequestMethods,
  HttpClientResponse,
  IJsonHttpClient
} from '#domain/clients';
import { ILogger } from '#domain/logging';
import {
  ClientResponseFactory,
  IPackageClient,
  PackageSourceType,
  PackageStatusFactory,
  TPackageClientRequest,
  TPackageClientResponse,
  TSemverSpec,
  VersionUtils,
  createSuggestions
} from '#domain/packages';
import {
  PackageDescriptorType,
  TPackageGitDescriptor,
  TPackagePathDescriptor,
} from '#domain/parsers';
import { CargoConfig, ICratesApiItem } from '#domain/providers/cargo';
import { throwUndefinedOrNull } from '@esm-test/guards';

export class CratesClient implements IPackageClient<null> {

  constructor(
    readonly config: CargoConfig,
    readonly jsonClient: IJsonHttpClient,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull("config", config);
    throwUndefinedOrNull("jsonClient", jsonClient);
    throwUndefinedOrNull("logger", logger);
  }

  async fetchPackage<TClientData>(
    request: TPackageClientRequest<TClientData>
  ): Promise<TPackageClientResponse> {
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
    if (gitDesc) return ClientResponseFactory.createGit();

    // fetch package suggestions from api
    const semverSpec = VersionUtils.parseSemver(requestedPackage.version);
    const url = `${this.config.apiUrl}${requestedPackage.name}/versions`;
    try {
      return await this.createRemotePackageDocument(url, request, semverSpec)
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
        )
      }

      throw errorResponse;
    }
  }

  async createRemotePackageDocument<TClientData>(
    url: string,
    request: TPackageClientRequest<TClientData>,
    semverSpec: TSemverSpec
  ): Promise<TPackageClientResponse> {

    const query = {};
    const headers = {};

    // fetch package from api
    const httpResponse = await this.jsonClient.request(
      HttpClientRequestMethods.get,
      url,
      query,
      headers
    );

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

    const responseVersions = httpResponse.data as ICratesApiItem;
    let rawVersions = responseVersions.versions
      .reverse()
      .map(p => p.num);

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