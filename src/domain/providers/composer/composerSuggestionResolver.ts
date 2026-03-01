import type { ILogger } from '#domain/logging';
import {
  type PackageClientRequest,
  type PackageClientResponse,
  type SemverSpec,
  PackageSourceType,
  VersionUtils,
  createSuggestions
} from '#domain/packages';
import type { ComposerConfig, PackagistClient } from '#domain/providers/composer';
import { throwUndefinedOrNull } from '@esm-test/guards';
import { coerce } from 'semver';

/**
 * Resolves package suggestions for Composer dependencies from the Packagist registry.
 */
export class ComposerSuggestionResolver {

  /**
   * Initializes a new instance of the ComposerSuggestionResolver class.
   * @param config The configuration for the Composer provider.
   * @param packagistClient The client used to interact with Packagist.
   * @param logger The logger for this resolver.
   */
  constructor(
    readonly config: ComposerConfig,
    readonly packagistClient: PackagistClient,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull('config', config);
    throwUndefinedOrNull('packagistClient', packagistClient);
    throwUndefinedOrNull('logger', logger);
  }

  /**
   * Resolves suggestions from the Packagist registry.
   * @param request The package client request.
   * @param semverSpec The parsed semver specification.
   * @returns A promise resolving to the package client response.
   */
  async fromPackagist(
    request: PackageClientRequest<null>,
    semverSpec: SemverSpec
  ): Promise<PackageClientResponse> {
    // fetch
    const requestPackage = request.parsedDependency.package;
    const jsonResponse = await this.packagistClient.get(requestPackage.name);

    // process response
    const versionRange = semverSpec.rawVersion;

    const resolved = {
      name: requestPackage.name,
      version: versionRange,
    };

    const responseStatus = {
      source: jsonResponse.source,
      status: jsonResponse.status,
    };

    const responseVersions = jsonResponse.data.packages[requestPackage.name];
    const rawVersions: string[] = responseVersions
      .map(x => coerce(x.version, VersionUtils.loosePrereleases).toString())
      .toSorted(VersionUtils.compareVersionsAndBuilds);

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