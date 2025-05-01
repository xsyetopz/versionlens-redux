import {
  addAppConfig,
  addCachingOptions,
  addEventScheduler,
  addFetchPackageSuggestionsUseCase,
  addFetchProjectSuggestionsUseCase,
  addFileSystemStorage,
  addFileWatcherDependencyCache,
  addGetDependencyChangesUseCase,
  addGetSuggestionProviderUseCase,
  addGetSuggestionsStatsUseCase,
  addHttpOptions,
  addShellCache,
  addSuggestionPackageCache,
  addSuggestionProviders,
  addUrlRequestCache,
} from '#domain';
import type { TConfigSectionResolver } from '#domain/configuration';
import type { IServiceCollection } from '#domain/di';
import { addLoggerFactory } from './logging/serviceFactory';

export function addDomainServices(
  services: IServiceCollection,
  configSection: string,
  configResolver: TConfigSectionResolver
) {
  addAppConfig(services, configSection, configResolver);
  addFileSystemStorage(services);
  addEventScheduler(services);

  // options
  addHttpOptions(services);
  addCachingOptions(services);

  // caches
  addFileWatcherDependencyCache(services);
  addSuggestionPackageCache(services);
  addShellCache(services);
  addUrlRequestCache(services);

  // logging
  addLoggerFactory(services);

  // providers
  addSuggestionProviders(services);

  // use cases
  addFetchProjectSuggestionsUseCase(services);
  addFetchPackageSuggestionsUseCase(services);
  addGetSuggestionProviderUseCase(services);
  addGetDependencyChangesUseCase(services);
  addGetSuggestionsStatsUseCase(services);
}