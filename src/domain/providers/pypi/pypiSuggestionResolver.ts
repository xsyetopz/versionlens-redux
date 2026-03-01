import type { ILogger } from '#domain/logging';
import {
  type PackageClientRequest,
  type PackageClientResponse,
  type PackageDependency,
  type SemverSpec,
  ClientResponseFactory,
  PackageSourceType,
  VersionUtils,
  createSuggestions
} from '#domain/packages';
import type { PackagePathDescriptor } from '#domain/parsers';
import { PypiConfig, PypiHttpClient } from '#domain/providers/pypi';
import { throwUndefinedOrNull } from '@esm-test/guards';
import { coerce } from 'semver';

/**
 * Resolves package suggestions for PyPi dependencies from various sources like registries, local paths, or Git.
 */
export class PypiSuggestionResolver {

  /**
   * Initializes a new instance of the PypiSuggestionResolver class.
   * @param config The configuration for the PyPi provider.
   * @param pypiHttpClient The client used to interact with PyPi.
   * @param logger The logger for this resolver.
   */
  constructor(
    readonly config: PypiConfig,
    readonly pypiHttpClient: PypiHttpClient,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull("config", config);
    throwUndefinedOrNull("pypiHttpClient", pypiHttpClient);
    throwUndefinedOrNull("logger", logger);
  }

  /**
   * Resolves suggestions for a local path dependency.
   * @param dependency The package dependency.
   * @param pathDesc The path descriptor.
   * @returns A promise resolving to the package client response.
   */
  fromPath(dependency: PackageDependency, pathDesc: PackagePathDescriptor): Promise<PackageClientResponse> {
    return ClientResponseFactory.createDirectory(
      dependency.package.name,
      dependency.package.path,
      pathDesc.path
    );
  }

  /**
   * Resolves suggestions for a Git dependency.
   * @returns The package client response for a Git dependency.
   */
  fromGit(): PackageClientResponse {
    return ClientResponseFactory.createGit();
  }

  /**
   * Resolves suggestions from the PyPi API.
   * @param request The package client request.
   * @param semverSpec The parsed semver specification.
   * @returns A promise resolving to the package client response.
   */
  async fromPypiApi(
    request: PackageClientRequest<null>,
    semverSpec: SemverSpec
  ): Promise<PackageClientResponse> {
    // fetch 
    const requestPackage = request.parsedDependency.package;
    const httpResponse = await this.pypiHttpClient.get(requestPackage.name);

    // process response
    const versionRange = semverSpec.rawVersion;
    const resolved = {
      name: requestPackage.name,
      version: versionRange,
    };

    // extract semver versions only
    const semverVersions = VersionUtils.filterSemverVersions(httpResponse.data)
      .map(x => coerce(x, VersionUtils.loosePrereleases).toString())
      .toSorted(VersionUtils.compareVersionsAndBuilds);

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
      responseStatus: ClientResponseFactory.mapStatusFromJsonResponse(httpResponse),
      type: semverSpec.type,
      resolved,
      suggestions,
    };
  }
}