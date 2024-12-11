import {
  addAppConfig,
  addCachingOptions,
  addFetchPackageSuggestionsUseCase,
  addFetchProjectSuggestionsUseCase,
  addFileSystemStorage,
  addFileWatcherDependencyCache,
  addGetDependencyChangesUseCase,
  addGetSuggestionProviderUseCase,
  addHttpOptions,
  addShellCache,
  addSuggestionPackageCache,
  addSuggestionProviders,
} from '#domain';
import type { TConfigSectionResolver } from '#domain/configuration';
import type { IServiceCollection } from '#domain/di';
import { addLoggingOptions, addWinstonLogger } from './logging/serviceFactory';

export function addDomainServices(
  services: IServiceCollection,
  configSection: string,
  configResolver: TConfigSectionResolver,
  defaultLogGroup: string
) {
  addAppConfig(services, configSection, configResolver);
  addFileSystemStorage(services);

  // options
  addHttpOptions(services);
  addCachingOptions(services);

  // caches
  addFileWatcherDependencyCache(services);
  addSuggestionPackageCache(services);
  addShellCache(services);

  // logging
  addLoggingOptions(services);
  addWinstonLogger(services, defaultLogGroup);

  // providers
  addSuggestionProviders(services);

  // use cases
  addFetchProjectSuggestionsUseCase(services);
  addFetchPackageSuggestionsUseCase(services);
  addGetSuggestionProviderUseCase(services);
  addGetDependencyChangesUseCase(services);
}