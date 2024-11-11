import { TConfigSectionResolver } from '#domain/configuration';
import { IServiceCollection } from '#domain/di';
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
  addLoggingOptions,
  addProcessesCache,
  addSuggestionPackageCache,
  addSuggestionProviders,
  addWinstonChannelLogger,
  addWinstonLogger
} from '#domain/services';

export function addDomainServices(
  services: IServiceCollection,
  configSection: string,
  configResolver: TConfigSectionResolver, 
  defaultLogGroup: string
) {

  addAppConfig(services, configSection, configResolver);

  addHttpOptions(services);

  addCachingOptions(services);

  addLoggingOptions(services);

  addFileSystemStorage(services);

  addSuggestionProviders(services);

  addFileWatcherDependencyCache(services);

  addSuggestionPackageCache(services);

  addProcessesCache(services);

  addFetchProjectSuggestionsUseCase(services);

  addFetchPackageSuggestionsUseCase(services);

  addGetSuggestionProviderUseCase(services);

  addGetDependencyChangesUseCase(services);

  addWinstonChannelLogger(services);

  addWinstonLogger(services, defaultLogGroup);

}