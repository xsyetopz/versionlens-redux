import { ILogger } from '#domain/logging';
import {
  DependencyCache,
  PackageResponse,
  SuggestionStatusText,
  SuggestionTypes
} from '#domain/packages';
import { ISuggestionProvider } from '#domain/providers';
import { FetchProjectSuggestions } from '#domain/useCases';
import { throwUndefinedOrNull } from '@esm-test/guards';
import { dirname } from 'node:path';

export class GetSuggestions {

  constructor(
    readonly fetchSuggestions: FetchProjectSuggestions,
    readonly dependencyCaches: DependencyCache[],
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull("fetchSuggestions", fetchSuggestions);
    throwUndefinedOrNull("dependencyCaches", dependencyCaches);
    throwUndefinedOrNull("logger", logger);
  }

  async execute(
    provider: ISuggestionProvider,
    projectPath: string,
    packageFilePath: string,
    includePrereleases: boolean
  ): Promise<PackageResponse[]> {

    // ensure the caching duration is up to date
    provider.config.caching.defrost();
    this.logger.debug(
      "caching duration is set to %s seconds",
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
    const suggestions = await this.fetchSuggestions.execute(
      provider,
      projectPath,
      packagePath,
      packageDeps
    );

    this.logger.info(
      "resolved %s %s package release and pre-release suggestions",
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