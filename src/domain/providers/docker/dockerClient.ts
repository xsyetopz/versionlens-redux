import type { HttpClientResponse, JsonClientResponse } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import {
  type IPackageClient,
  type TPackageClientRequest,
  type TPackageClientResponse,
  type TPackageResource,
  ClientResponseFactory,
  createSuggestions,
  PackageSourceType,
  PackageStatusFactory,
  PackageVersionType,
  SuggestionCategory,
  UpdateableFactory,
  VersionUtils
} from '#domain/packages';
import { type PackagePathDescriptor, PackageDescriptorType } from '#domain/parsers';
import {
  type DockerApiTagResult,
  type DockerConfig,
  type DockerHubClient,
  createVersionMapper
} from '#domain/providers/docker';
import { throwUndefinedOrNull } from '@esm-test/guards';

export class DockerClient implements IPackageClient<null> {

  constructor(
    readonly config: DockerConfig,
    readonly dockerHubClient: DockerHubClient,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull("config", config);
    throwUndefinedOrNull("dockerHubClient", dockerHubClient);
    throwUndefinedOrNull("logger", logger);
  }

  async fetchPackage(request: TPackageClientRequest<null>): Promise<TPackageClientResponse> {
    const dependency = request.parsedDependency
    const requestedPackage = dependency.package;

    // process build context path types
    if (dependency.descriptors.hasType('path')) {
      const pathDesc = dependency.descriptors.getType<PackagePathDescriptor>(
        PackageDescriptorType.path
      );
      return await ClientResponseFactory.createDirectory(
        requestedPackage.name,
        requestedPackage.path,
        pathDesc.path,
        PackageSourceType.File
      );
    }

    // ignore FROMs composed using arguments
    if (requestedPackage.name.includes('$') || requestedPackage.version.includes('$')) {
      return ClientResponseFactory.createNotSupported()
    }

    try {
      // fetch from the api
      let namespace = 'library'
      let repo = requestedPackage.name
      if (requestedPackage.name.includes('/')) {
        [namespace, repo] = requestedPackage.name.split('/');
      }

      return await this.fetch(requestedPackage, repo, namespace);
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
        );
      }

      throw errorResponse;
    }
  }

  async fetch(pkg: TPackageResource, repo: string, namespace: string): Promise<TPackageClientResponse> {
    const jsonResponse: JsonClientResponse = await this.dockerHubClient.get(repo, namespace)

    // process api response
    const results = jsonResponse.data as DockerApiTagResult[]

    // map docker tags to semver
    const { versionMap, tagMap, releases, latest } = createVersionMapper(results);

    const tagged = Object.keys(tagMap);
    let versionRange = pkg.version || latest || '';
    if (tagged.includes(versionRange)) versionRange = tagMap[versionRange];

    // analyse suggestions
    let suggestions = createSuggestions(versionRange, releases, []);

    const buildSuggestion = suggestions.find(x => x.category === SuggestionCategory.Build);
    if (!buildSuggestion && versionMap[versionRange] !== undefined) {
      suggestions.push(UpdateableFactory.createBuildUpdateable(''))
    }

    // map suggestion text back to the real docker tags
    for (const suggestion of suggestions) {
      switch (suggestion.category) {
        case SuggestionCategory.Build:
          const uniqueVersions: Record<string, boolean> = {}
          if (suggestion.version.length > 0) {
            const buildVersions = suggestion.version.split(',')
            for (const v of buildVersions) {
              const tags = versionMap[v]
              for (const tag of tags) uniqueVersions[tag] = true
            }
          }

          const additionalTags = versionMap[versionRange] ?? []
          for (const tag of additionalTags) uniqueVersions[tag] = true

          suggestion.version = Object.keys(uniqueVersions)
            .toSorted((a, b) => {
              if (a === 'latest') return -1
              if (b === 'latest') return 1
              return a > b
                ? 1
                : a < b ? -1 : 0
            })
            .join(',')
          break;
        case SuggestionCategory.Latest:
        case SuggestionCategory.Match:
        case SuggestionCategory.Updateable:
          suggestion.version = VersionUtils.stripBuild(suggestion.version)!
      }
    }

    // return PackageDocument
    const resolved = {
      name: pkg.name,
      version: versionRange,
    };
    return {
      source: PackageSourceType.Registry,
      responseStatus: ClientResponseFactory.mapStatusFromJsonResponse(jsonResponse),
      type: PackageVersionType.Version,
      resolved,
      suggestions,
    };
  }

}