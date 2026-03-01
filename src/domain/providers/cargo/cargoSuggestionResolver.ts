import type { ILogger } from '#domain/logging';
import {
  type PackageClientRequest,
  type PackageClientResponse,
  type SemverSpec,
  ClientResponseFactory,
  PackageDependency,
  PackageSourceType,
  VersionUtils,
  createSuggestions
} from '#domain/packages';
import type { PackagePathDescriptor } from '#domain/parsers';
import type { CargoConfig, CratesClient } from '#domain/providers/cargo';
import { throwUndefinedOrNull } from '@esm-test/guards';

/**
 * Resolves package suggestions for Cargo dependencies from various sources like Crates.io, local paths, or Git.
 */
export class CargoSuggestionResolver {

  /**
   * Initializes a new instance of the CargoSuggestionResolver class.
   * @param config The configuration for the Cargo provider.
   * @param cratesClient The client used to interact with Crates.io.
   * @param logger The logger for this resolver.
   */
  constructor(
    readonly config: CargoConfig,
    readonly cratesClient: CratesClient,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull('config', config);
    throwUndefinedOrNull('cratesClient', cratesClient);
    throwUndefinedOrNull('logger', logger);
  }

  /**
   * Resolves suggestions from the Crates.io registry.
   * @param request The package client request.
   * @param semverSpec The parsed semver specification.
   * @returns A promise resolving to the package client response.
   */
  async fromCrates(
    request: PackageClientRequest<null>,
    semverSpec: SemverSpec
  ): Promise<PackageClientResponse> {
    // fetch
    const requestPackage = request.parsedDependency.package;
    const jsonResponse = await this.cratesClient.get(requestPackage.name);

    // process response
    const versionRange = semverSpec.rawVersion;
    const resolved = {
      name: requestPackage.name,
      version: versionRange,
    };

    // filter and sort versions
    const rawVersions = jsonResponse.data.versions
      .filter(p => p.yanked === false)
      .map(p => p.num)
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
      responseStatus: ClientResponseFactory.mapStatusFromJsonResponse(jsonResponse),
      type: semverSpec.type,
      resolved,
      suggestions,
    };
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

}