import type { ILogger } from '#domain/logging';
import {
  type PackageClientRequest,
  type PackageClientResponse,
  type PackageSuggestion,
  type SemverSpec,
  ClientResponseFactory,
  PackageSourceType,
  PackageStatusFactory,
  VersionUtils,
  createSuggestions
} from '#domain/packages';
import type { DubConfig, DubJsonClient } from '#domain/providers/dub';
import { throwUndefinedOrNull } from '@esm-test/guards';
import { valid } from 'semver';

/**
 * Resolves package suggestions for Dub dependencies from the Dub registry.
 */
export class DubSuggestionResolver {

  /**
   * Initializes a new instance of the DubSuggestionResolver class.
   * @param config The configuration for the Dub provider.
   * @param dubJsonClient The client used to interact with the Dub registry.
   * @param logger The logger for this resolver.
   */
  constructor(
    readonly config: DubConfig,
    readonly dubJsonClient: DubJsonClient,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull('config', config);
    throwUndefinedOrNull('dubJsonClient', dubJsonClient);
    throwUndefinedOrNull('logger', logger);
  }

  /**
   * Resolves suggestions from the Dub API.
   * @param request The package client request.
   * @param semverSpec The parsed semver specification.
   * @returns A promise resolving to the package client response.
   */
  async fromDubApi(
    request: PackageClientRequest<null>,
    semverSpec: SemverSpec
  ): Promise<PackageClientResponse> {
    const requestedPackage = request.parsedDependency.package;
    const jsonResponse = await this.dubJsonClient.get(requestedPackage.name);

    // process response
    const versionRange = semverSpec.rawVersion;

    const resolved = {
      name: requestedPackage.name,
      version: versionRange,
    };

    // seperate versions to releases and prereleases
    const { releases, prereleases } = VersionUtils.splitReleasesFromArray(
      jsonResponse.data,
      this.config.prereleaseTagFilter
    );

    // analyse suggestions
    const suggestions = parseSuggestions(
      versionRange,
      releases,
      prereleases
    );

    return {
      source: PackageSourceType.Registry,
      responseStatus: ClientResponseFactory.mapStatusFromJsonResponse(jsonResponse),
      type: semverSpec.type,
      resolved,
      suggestions,
    };
  }

}

/**
 * Parses release and prerelease versions into package suggestions.
 * @param versionRange The current version range.
 * @param releases The list of release versions.
 * @param prereleases The list of prerelease versions.
 * @returns An array of package suggestions.
 */
function parseSuggestions(
  versionRange: string,
  releases: string[],
  prereleases: string[]
): Array<PackageSuggestion> {
  if (releases.length === 0) {
    return [PackageStatusFactory.createNoMatchStatus()]
  }

  const latestRelease = releases[releases.length - 1];
  const isValid = valid(versionRange.replace('~>', ''));

  // checks if this is a repo version
  if (!isValid && versionRange.startsWith('~') && latestRelease === versionRange) {
    return [PackageStatusFactory.createMatchesLatestStatus(versionRange)]
  }

  return createSuggestions(
    versionRange,
    releases,
    prereleases
  );
}