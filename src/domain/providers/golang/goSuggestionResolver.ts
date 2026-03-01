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
import type { GoConfig, GoHttpClient } from '#domain/providers/golang';
import { throwUndefinedOrNull } from '@esm-test/guards';
import { coerce, compareLoose } from 'semver';

/**
 * Resolves package suggestions for Go dependencies from various sources like Go proxies, local paths, or Git.
 */
export class GoSuggestionResolver {

  /**
   * Initializes a new instance of the GoSuggestionResolver class.
   * @param config The configuration for the Go provider.
   * @param goHttpClient The client used to interact with the Go proxy.
   * @param logger The logger for this resolver.
   */
  constructor(
    readonly config: GoConfig,
    readonly goHttpClient: GoHttpClient,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull('config', config);
    throwUndefinedOrNull('goHttpClient', goHttpClient);
    throwUndefinedOrNull('logger', logger);
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
   * Resolves suggestions from the Go API (proxy).
   * @param request The package client request.
   * @param semverSpec The parsed semver specification.
   * @returns A promise resolving to the package client response.
   */
  async fromGoApi<TClientData>(
    request: PackageClientRequest<TClientData>,
    semverSpec: SemverSpec
  ): Promise<PackageClientResponse> {
    // fetch
    const requestPackage = request.parsedDependency.package;
    const httpResponse = await this.goHttpClient.get(requestPackage.name);

    // process response
    const { data } = httpResponse;
    const versionRange = semverSpec.rawVersion;

    const resolved = {
      name: requestPackage.name,
      version: versionRange,
    };

    const responseStatus = {
      source: httpResponse.source,
      status: httpResponse.status,
    };

    // sort versions
    const rawVersions = data.versions.toSorted(VersionUtils.compareVersionsAndBuilds)

    // extract semver versions only
    const semverVersions = VersionUtils.filterSemverVersions(rawVersions)
      .map(x => coerce(x, VersionUtils.loosePrereleases).toString());

    // seperate versions to releases and prereleases
    const { releases, prereleases } = VersionUtils.splitReleasesFromArray(
      semverVersions.sort(compareLoose),
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