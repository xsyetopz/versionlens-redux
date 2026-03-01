import type { ILogger } from '#domain/logging';
import {
  type PackageResponse,
  DependencyCache,
  SuggestionStatusText,
  SuggestionTypes
} from '#domain/packages';
import type { ISuggestionProvider } from '#domain/providers';
import type { FetchPackages } from '#domain/useCases';
import { throwUndefinedOrNull } from '@esm-test/guards';
import { dirname } from 'node:path';

/**
 * Use case for retrieving version suggestions for a package file.
 */
export class GetSuggestions {

  /**
   * Initializes a new instance of the GetSuggestions class.
   * @param fetchPackages The use case for batch fetching package data.
   * @param dependencyCaches The caches to search for parsed dependencies.
   * @param logger The logger to use.
   */
  constructor(
    readonly fetchPackages: FetchPackages,
    readonly dependencyCaches: DependencyCache[],
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull("fetchPackages", fetchPackages);
    throwUndefinedOrNull("dependencyCaches", dependencyCaches);
    throwUndefinedOrNull("logger", logger);
  }

  /**
   * Executes the get suggestions use case.
   * @param provider The suggestion provider for the file.
   * @param projectPath The path to the project.
   * @param packageFilePath The path to the package file.
   * @param includePrereleases Whether to include prerelease versions in suggestions.
   * @returns A promise resolving to an array of package responses.
   */
  async execute(
    provider: ISuggestionProvider,
    projectPath: string,
    packageFilePath: string,
    includePrereleases: boolean
  ): Promise<PackageResponse[]> {

    // ensure the caching duration is up to date
    provider.config.caching.defrost();
    this.logger.debug(
      "caching duration is set to {duration} seconds",
      provider.config.caching.duration / 1000
    );

    // get the document dependencies
    const packageDeps = DependencyCache.getDependenciesWithFallback(
      provider.name,
      packageFilePath,
      ...this.dependencyCaches
    );

    // fetch the package suggestions
    const packagePath = dirname(packageFilePath);
    const suggestions = await this.fetchPackages.execute(
      provider,
      projectPath,
      packagePath,
      packageDeps
    );

    this.logger.info(
      "resolved {suggestionCount} {providerName} package release and pre-release suggestions",
      suggestions.length,
      provider.name
    );

    // return without preleases
    if (includePrereleases === false) {
      return suggestions.filter(
        function (response) {
          const { suggestion } = response;
          return suggestion
            && (
              (suggestion.type & SuggestionTypes.prerelease) === 0
              || suggestion.name.includes(SuggestionStatusText.LatestIsPrerelease)
            );
        }
      )
    }

    // return all suggestions
    return suggestions;
  }

}