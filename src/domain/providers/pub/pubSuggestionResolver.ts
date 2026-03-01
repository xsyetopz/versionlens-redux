import type { ILogger } from '#domain/logging';
import {
  type PackageClientResponse,
  type PackageDependency,
  type SemverSpec,
  ClientResponseFactory,
  PackageSourceType,
  VersionUtils,
  createSuggestions
} from '#domain/packages';
import type { PackagePathDescriptor } from '#domain/parsers';
import type { PubConfig, PubJsonClient } from '#domain/providers/pub';
import { throwUndefinedOrNull } from '@esm-test/guards';

/**
 * Resolves package suggestions for Pub dependencies from various sources like registries, local paths, or Git.
 */
export class PubSuggestionResolver {

  /**
   * Initializes a new instance of the PubSuggestionResolver class.
   * @param config The configuration for the Pub provider.
   * @param pubJsonClient The client used to interact with the Pub registry.
   * @param logger The logger for this resolver.
   */
  constructor(
    readonly config: PubConfig,
    readonly pubJsonClient: PubJsonClient,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull("config", config);
    throwUndefinedOrNull("pubJsonClient", pubJsonClient);
    throwUndefinedOrNull("logger", logger);
  }

  /**
   * Resolves suggestions for a local path dependency.
   * @param dep The package dependency.
   * @param pathDesc The path descriptor.
   * @returns The package client response for a directory dependency.
   */
  fromPath(dep: PackageDependency, pathDesc: PackagePathDescriptor) {
    return ClientResponseFactory.createDirectory(
      dep.package.name,
      dep.package.path,
      pathDesc.path
    );
  }

  /**
   * Resolves suggestions for a Git dependency.
   * @returns The package client response for a Git dependency.
   */
  fromGit() {
    return ClientResponseFactory.createGit();
  }

  /**
   * Resolves suggestions from the Pub API.
   * @param url The API URL for the package.
   * @param packageName The name of the package.
   * @param semverSpec The parsed semver specification.
   * @returns A promise resolving to the package client response.
   */
  async fromPubApi(
    url: string,
    packageName: string,
    semverSpec: SemverSpec
  ): Promise<PackageClientResponse> {
    // fetch
    const jsonResponse = await this.pubJsonClient.get(url);

    // process response
    const versionRange = semverSpec.rawVersion;

    const resolved = {
      name: packageName,
      version: versionRange,
    };

    // sort versions
    const rawVersions = jsonResponse.data.versions
      .toSorted(VersionUtils.compareVersionsAndBuilds);

    // seperate versions to releases and prereleases
    const { releases, prereleases } = VersionUtils.splitReleasesFromArray(
      rawVersions,
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