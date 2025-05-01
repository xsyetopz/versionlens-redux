import { MemoryCache } from '#domain/caching';
import type { ILogger } from '#domain/logging';
import { type DependencyCache, SuggestionCategory, SuggestionTypes } from '#domain/packages';
import type { ISuggestionProvider } from '#domain/providers';
import type { GetSuggestions } from '#domain/useCases';
import { Disposable } from '#domain/utils';
import { throwUndefinedOrNull } from '@esm-test/guards';

export type SuggestionsStats = {
  filePath: string
  providerName: string
  noMatches: number
  updates: number
  errors: number
}

export class GetSuggestionsStats extends Disposable {
  readonly cache = new MemoryCache<SuggestionsStats[]>('stats-cache')

  constructor(
    readonly providers: ISuggestionProvider[],
    readonly dependencyCache: DependencyCache,
    readonly getSuggestions: GetSuggestions,
    readonly logger: ILogger
  ) {
    super();
    throwUndefinedOrNull('providers', providers);
    throwUndefinedOrNull('dependencyCache', dependencyCache);
    throwUndefinedOrNull('getSuggestions', getSuggestions);
    throwUndefinedOrNull('logger', logger);
  }

  async execute(useCache: boolean): Promise<SuggestionsStats[]> {
    if (useCache) {
      const cached = this.cache.get('stats');
      if (cached) return cached;
    }

    const providerProjectFiles = this.providers.flatMap(
      provider => {
        const filePaths = this.dependencyCache.getFilePaths(provider.name);
        return filePaths.map(filePath => ({ provider, filePath }));
      }
    );

    // create promise queue
    const suggestionPromises = providerProjectFiles.map(
      ({ provider, filePath }) => {
        this.logger.debug("queueing suggestion stats for {PackageFilePath}", filePath)
        return this.getSuggestions.execute(
          provider,
          filePath,
          filePath,
          false
        )
      }
    );

    // parallel fetch promises
    const resolvedSuggestions = await Promise.all(suggestionPromises)

    const stats: SuggestionsStats[] = []
    for (const suggestions of resolvedSuggestions) {
      const statuses = suggestions
        .filter(x => x.suggestion.type === SuggestionTypes.status)
        .map(x => x.suggestion);

      let noMatches = 0;
      let updates = 0;
      let errors = 0;
      for (const status of statuses) {
        const cat = status.category;
        if (cat === SuggestionCategory.NoMatch)
          noMatches++;
        else if (cat === SuggestionCategory.Error)
          errors++;
        else if (cat !== SuggestionCategory.Latest && cat !== SuggestionCategory.Directory)
          updates++;
      }

      if (noMatches + updates + errors > 0) {
        const firstSuggestion = suggestions[0];
        stats.push({
          filePath: firstSuggestion.parsedDependency.package.path,
          providerName: firstSuggestion.providerName,
          noMatches,
          errors,
          updates
        });
      }
    }

    return this.cache.set('stats', stats);
  }

}