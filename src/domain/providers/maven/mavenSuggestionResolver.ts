import type { ILogger } from '#domain/logging';
import {
  type PackageClientRequest,
  type PackageClientResponse,
  type SemverSpec,
  ClientResponseFactory,
  PackageSourceType,
  VersionUtils,
  createSuggestions
} from '#domain/packages';
import type {
  MavenClientData,
  MavenConfig,
  MavenHttpClient
} from '#domain/providers/maven';
import { throwUndefinedOrNull } from '@esm-test/guards';
import { valid } from 'semver';

/**
 * Resolves package suggestions for Maven dependencies from various Maven registries.
 */
export class MavenSuggestionResolver {

  /**
   * Initializes a new instance of the MavenSuggestionResolver class.
   * @param config The configuration for the Maven provider.
   * @param mavenHttpClient The client used to interact with Maven repositories.
   * @param logger The logger for this resolver.
   */
  constructor(
    readonly config: MavenConfig,
    readonly mavenHttpClient: MavenHttpClient,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull('config', config);
    throwUndefinedOrNull('mavenHttpClient', mavenHttpClient);
    throwUndefinedOrNull('logger', logger);
  }

  /**
   * Resolves suggestions from the Maven API (repositories).
   * @param repos The list of repository URLs to search.
   * @param request The package client request.
   * @param semverSpec The parsed semver specification.
   * @returns A promise resolving to the package client response.
   */
  async fromMavenApi(
    repos: string[],
    request: PackageClientRequest<MavenClientData>,
    semverSpec: SemverSpec
  ): Promise<PackageClientResponse> {
    // fetch
    const requestedPackage = request.parsedDependency.package;
    const httpResponse = await this.mavenHttpClient.get(requestedPackage.name, repos);

    // process response
    const source = PackageSourceType.Registry;
    const versionRange = semverSpec.rawVersion;

    // extract semver versions only
    const semverVersions = VersionUtils.filterSemverVersions(httpResponse.data)
      .filter(x => !!valid(x))
      .toSorted(VersionUtils.compareVersionsAndBuilds);

    // seperate versions to releases and prereleases
    const { releases, prereleases } = VersionUtils.splitReleasesFromArray(
      semverVersions,
      this.config.prereleaseTagFilter
    );

    const resolved = {
      name: requestedPackage.name,
      version: versionRange,
    };

    // analyse suggestions
    const suggestions = createSuggestions(
      versionRange,
      releases,
      prereleases
    );

    return {
      source,
      responseStatus: ClientResponseFactory.mapStatusFromJsonResponse(httpResponse),
      type: semverSpec.type,
      resolved,
      suggestions,
    };
  }

}