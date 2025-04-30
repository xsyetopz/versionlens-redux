import type { ILogger } from '#domain/logging';
import {
  type PackageClientResponse,
  type PackageDependency,
  type PackageResource,
  ClientResponseFactory,
  createSuggestions,
  PackageSourceType,
  PackageVersionType,
  SuggestionCategory,
  UpdateableFactory,
  VersionUtils
} from '#domain/packages';
import {
  type PackagePathDescriptor,
  PackageDescriptorType,
  PackageRegistryDescriptor
} from '#domain/parsers';
import {
  type DockerClientResponse,
  type DockerConfig,
  type DockerHubClient,
  type MicrosoftDockerClient,
  createVersionMapper,
  findSimilarBuild
} from '#domain/providers/docker';
import { throwUndefinedOrNull } from '@esm-test/guards';

export class DockerSuggestionResolver {

  constructor(
    readonly config: DockerConfig,
    readonly dockerHubClient: DockerHubClient,
    readonly microsoftDockerClient: MicrosoftDockerClient,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull('config', config);
    throwUndefinedOrNull('dockerHubClient', dockerHubClient);
    throwUndefinedOrNull('microsoftDockerClient', microsoftDockerClient);
    throwUndefinedOrNull('logger', logger);
  }

  async fromPath(dependency: PackageDependency) {
    const requestedPackage = dependency.package;

    // process build context path types
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

  async fromRegistry(dependency: PackageDependency): Promise<PackageClientResponse> {
    const requestedPackage = dependency.package;
    const registryDesc = dependency.descriptors.getType<PackageRegistryDescriptor>('registry');
    const registry = registryDesc?.registry ?? ''
    let namespace = 'library';
    let repo = requestedPackage.name;
    if (requestedPackage.name.includes('/')) {
      [namespace, repo] = requestedPackage.name.split('/');
    }

    const jsonResponse = registry.length > 0
      ? await this.microsoftDockerClient.get(repo, namespace)
      : await this.dockerHubClient.get(repo, namespace);

    return this.parseResponse(requestedPackage, jsonResponse);
  }

  private async parseResponse(pkg: PackageResource, jsonResponse: DockerClientResponse): Promise<PackageClientResponse> {
    // map docker tags to semver
    const { versionMap, tagMap, releases, latest } = createVersionMapper(jsonResponse.data);

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
          suggestion.version = VersionUtils.stripBuild(suggestion.version)!
          break;
        case SuggestionCategory.Updateable:
          const suggestionBuilds = versionMap[suggestion.version] ?? []
          const similarBuild = findSimilarBuild(pkg.version, suggestionBuilds)
          suggestion.version = similarBuild !== null
            ? similarBuild
            : VersionUtils.stripBuild(suggestion.version)!
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