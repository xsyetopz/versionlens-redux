import type { HttpClientResponse, IHttpClient } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import {
  type IPackageClient,
  type TPackageClientRequest,
  type TPackageClientResponse,
  type TSemverSpec,
  ClientResponseFactory,
  PackageSourceType,
  PackageStatusFactory,
  VersionUtils,
  createSuggestions
} from '#domain/packages';
import {
  type TPackageGitDescriptor,
  type TPackagePathDescriptor,
  PackageDescriptorType,
  XmlDoc
} from '#domain/parsers';
import { PypiConfig } from '#domain/providers/pypi';
import { throwUndefinedOrNull } from '@esm-test/guards';
import semver from 'semver';

export class PypiClient implements IPackageClient<null> {

  constructor(
    readonly config: PypiConfig,
    readonly httpClient: IHttpClient,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull("config", config);
    throwUndefinedOrNull("httpClient", httpClient);
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
      return await ClientResponseFactory.createDirectory(
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
    const url = this.config.apiUrl.replace('{name}', requestedPackage.name);
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

    // fetch package from api
    const httpResponse = await this.httpClient.get(url);

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

    const xmlDoc = new XmlDoc()
    xmlDoc.parse(httpResponse.data)
    const rawVersions = xmlDoc.findExactPaths("rss.channel.item.title")
      .map(x => x.text)
      .reverse();

    // extract semver versions only
    const { coerce } = semver;
    const semverVersions = VersionUtils.filterSemverVersions(rawVersions)
      .map(x => coerce(x, VersionUtils.loosePrereleases).toString());

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