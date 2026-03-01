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
import type { ConfigSectionResolver } from '#domain/configuration';
import type { IServiceCollection } from '#domain/di';
import { addLoggerFactory } from './logging/serviceFactory';

/**
 * Configures the service collection by adding all domain-level services.
 * @param services The service collection to add to.
 * @param configSection The name of the configuration section.
 * @param configResolver The function to resolve configuration values.
 */
export function addDomainServices(
  services: IServiceCollection,
  configSection: string,
  configResolver: ConfigSectionResolver
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