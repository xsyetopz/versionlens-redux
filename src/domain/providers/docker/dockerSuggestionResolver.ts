import type { ILogger } from '#domain/logging';
import {
  type PackageClientResponse,
  type PackageDependency,
  type PackageResource,
  ClientResponseFactory,
  createSuggestions,
  PackageSourceType,
  PackageStatusFactory,
  PackageVersionType,
  SuggestionCategory,
  UpdateableFactory
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

/**
 * Resolves package suggestions for Docker images from various sources like Docker Hub, Microsoft Docker Registry, or local paths.
 */
export class DockerSuggestionResolver {

  /**
   * Initializes a new instance of the DockerSuggestionResolver class.
   * @param config The configuration for the Docker provider.
   * @param dockerHubClient The client for Docker Hub.
   * @param microsoftDockerClient The client for Microsoft Docker Registry.
   * @param logger The logger for this resolver.
   */
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

  /**
   * Resolves suggestions for a local path dependency (e.g., build context).
   * @param dependency The package dependency.
   * @returns A promise resolving to the package client response.
   */
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

  /**
   * Resolves suggestions from a Docker registry (Docker Hub or Microsoft).
   * @param dependency The package dependency.
   * @returns A promise resolving to the package client response.
   */
  async fromRegistry(dependency: PackageDependency): Promise<PackageClientResponse> {
    const requestedPackage = dependency.package;
    const registryDesc = dependency.descriptors.getType<PackageRegistryDescriptor>('registry');
    const registry = registryDesc?.registry ?? '';
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

  /**
   * Parses the response from the Docker registry and creates suggestions.
   * @param pkg The package resource.
   * @param jsonResponse The Docker client response.
   * @returns A promise resolving to the package client response.
   */
  private async parseResponse(pkg: PackageResource, jsonResponse: DockerClientResponse): Promise<PackageClientResponse> {
    // map docker tags to semver
    const { versionMap, tagMap, releases, latest } = createVersionMapper(jsonResponse.data);

    const tagged = Object.keys(tagMap);
    let versionRange = pkg.version || latest || '';
    const tagExists = tagged.includes(versionRange);
    if (tagExists) versionRange = tagMap[versionRange];

    // analyse suggestions
    const suggestions = createSuggestions(versionRange, releases, []);

    // ensure we dont match non-existing versions
    if (tagExists === false && pkg.version !== '') {
      suggestions[0] = PackageStatusFactory.createNoMatchStatus();
    }

    const buildSuggestion = suggestions.find(x => x.category === SuggestionCategory.Build);
    if (!buildSuggestion && versionMap[versionRange] !== undefined) {
      suggestions.push(UpdateableFactory.createBuildUpdateable(''));
    }

    // map suggestion text back to the real docker tags
    for (const suggestion of suggestions) {
      switch (suggestion.category) {
        case SuggestionCategory.Build:
          const uniqueVersions: Record<string, boolean> = {};
          if (suggestion.version.length > 0) {
            const buildVersions = suggestion.version.split(',');
            for (const v of buildVersions) {
              const tags = versionMap[v];
              for (const tag of tags) uniqueVersions[tag] = true;
            }
          }

          const additionalTags = versionMap[versionRange] ?? [];
          for (const tag of additionalTags) uniqueVersions[tag] = true;

          suggestion.version = Object.keys(uniqueVersions)
            .toSorted((a, b) => {
              if (a === 'latest') return -1
              if (b === 'latest') return 1
              return a > b
                ? 1
                : a < b ? -1 : 0
            })
            .join(',');
          break;
        case SuggestionCategory.Latest:
        case SuggestionCategory.Match:
          if (suggestion.version.includes('+')) {
            const tags = versionMap[suggestion.version];
            suggestion.version = tags[tags.length - 1];
          }
          break;
        case SuggestionCategory.Updateable:
          const suggestionBuilds = versionMap[suggestion.version] ?? [];
          const similarBuild = findSimilarBuild(pkg.version, suggestionBuilds);
          if (similarBuild !== null)
            suggestion.version = similarBuild;
          else if (suggestion.version.includes('+'))
            suggestion.version = suggestion.name === 'latest' && tagMap['latest']
              ? 'latest'
              : suggestionBuilds[suggestionBuilds.length - 1];
          break;
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