import { DomainServiceName, type IDomainServices } from '#domain';
import { CachingOptions, MemoryExpiryCache } from '#domain/caching';
import { HttpOptions } from '#domain/clients';
import { Config, type TConfigSectionResolver } from '#domain/configuration';
import type { IServiceCollection } from '#domain/di';
import { DependencyCache, PackageCache } from '#domain/packages';
import { importSuggestionProviders } from '#domain/providers';
import { FileSystemStorage } from '#domain/storage';
import {
  FetchPackage,
  FetchPackages,
  GetDependencyChanges,
  GetSuggestionProvider,
  GetSuggestionsStats
} from '#domain/useCases';
import { EventScheduler } from '#domain/utils';

export function addAppConfig(
  services: IServiceCollection,
  configSection: string,
  configSectionResolver: TConfigSectionResolver
) {
  services.addSingleton(
    DomainServiceName.appConfig,
    () => new Config(configSectionResolver, configSection.toLowerCase())
  )
}

export function addHttpOptions(services: IServiceCollection) {
  services.addSingleton(
    DomainServiceName.httpOptions,
    (container: IDomainServices) => new HttpOptions(container.appConfig, 'http')
  )
}

export function addCachingOptions(services: IServiceCollection) {
  services.addSingleton(
    DomainServiceName.cachingOptions,
    (container: IDomainServices) => new CachingOptions(container.appConfig, 'caching')
  )
}

export function addFileSystemStorage(services: IServiceCollection) {
  services.addSingleton(DomainServiceName.storage, new FileSystemStorage());
}

export async function addSuggestionProviders(services: IServiceCollection) {
  services.addSingleton(
    DomainServiceName.suggestionProviders,
    async (container: IDomainServices) => {
      return await importSuggestionProviders(
        container.serviceProvider,
        container.providerNames,
        container.loggerFactory.create(importSuggestionProviders.name)
      )
    }
  )
}

export function addFileWatcherDependencyCache(services: IServiceCollection) {
  services.addSingleton(
    DomainServiceName.fileWatcherDependencyCache,
    (container: IDomainServices) => new DependencyCache(container.providerNames)
  );
}

export function addSuggestionPackageCache(services: IServiceCollection) {
  services.addSingleton(
    DomainServiceName.packageCache,
    (container: IDomainServices) => new PackageCache(container.providerNames)
  );
}

export function addShellCache(services: IServiceCollection) {
  const serviceName = DomainServiceName.shellCache;
  services.addSingleton(serviceName, new MemoryExpiryCache(serviceName));
}

export function addUrlRequestCache(services: IServiceCollection) {
  const serviceName = DomainServiceName.urlRequestCache;
  services.addSingleton(serviceName, new MemoryExpiryCache(serviceName)
  );
}

export function addGetSuggestionProviderUseCase(services: IServiceCollection) {
  const serviceName = DomainServiceName.GetSuggestionProvider;
  services.addSingleton(
    serviceName,
    (container: IDomainServices) => new GetSuggestionProvider(container.suggestionProviders)
  );
}

export function addGetDependencyChangesUseCase(services: IServiceCollection) {
  const serviceName = DomainServiceName.getDependencyChanges;
  services.addSingleton(
    serviceName,
    (container: IDomainServices) =>
      new GetDependencyChanges(
        container.storage,
        container.fileWatcherDependencyCache,
        container.loggerFactory.create(serviceName)
      )
  );
}

export function addFetchProjectSuggestionsUseCase(services: IServiceCollection) {
  const serviceName = DomainServiceName.fetchPackages;
  services.addSingleton(
    serviceName,
    (container: IDomainServices) =>
      new FetchPackages(
        container.fetchPackage,
        container.loggerFactory.create(serviceName)
      )
  );
}

export function addFetchPackageSuggestionsUseCase(services: IServiceCollection) {
  const serviceName = DomainServiceName.fetchPackage;
  services.addSingleton(
    serviceName,
    (container: IDomainServices) =>
      new FetchPackage(
        container.packageCache,
        container.loggerFactory.create(serviceName)
      )
  );
}

export function addGetSuggestionsStatsUseCase(services: IServiceCollection) {
  const serviceName = DomainServiceName.getSuggestionsStats;
  services.addSingleton(
    serviceName,
    (container: IDomainServices) =>
      new GetSuggestionsStats(
        container.suggestionProviders,
        container.fileWatcherDependencyCache,
        container.getSuggestions,
        container.loggerFactory.create(serviceName)
      )
  );
}

export function addEventScheduler(services: IServiceCollection) {
  const serviceName = DomainServiceName.eventScheduler;
  services.addSingleton(
    serviceName,
    () => new EventScheduler()
  );
}