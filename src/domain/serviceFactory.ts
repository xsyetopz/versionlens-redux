import type { IDomainServices } from '#domain';
import { CachingOptions, MemoryExpiryCache } from '#domain/caching';
import { HttpOptions } from '#domain/clients';
import { type TConfigSectionResolver, Config } from '#domain/configuration';
import type { IServiceCollection } from '#domain/di';
import { DependencyCache, PackageCache } from '#domain/packages';
import { importSuggestionProviders } from '#domain/providers';
import { FileSystemStorage } from '#domain/storage';
import {
  FetchPackage,
  FetchPackages,
  GetDependencyChanges,
  GetSuggestionProvider
} from '#domain/useCases';
import { nameOf } from '#domain/utils';

export function addAppConfig(
  services: IServiceCollection,
  configSection: string,
  configSectionResolver: TConfigSectionResolver
) {
  services.addSingleton(
    nameOf<IDomainServices>().appConfig,
    () => new Config(configSectionResolver, configSection.toLowerCase())
  )
}

export function addHttpOptions(services: IServiceCollection) {
  services.addSingleton(
    nameOf<IDomainServices>().httpOptions,
    (container: IDomainServices) => new HttpOptions(container.appConfig, 'http')
  )
}

export function addCachingOptions(services: IServiceCollection) {
  services.addSingleton(
    nameOf<IDomainServices>().cachingOptions,
    (container: IDomainServices) => new CachingOptions(container.appConfig, 'caching')
  )
}

export function addFileSystemStorage(services: IServiceCollection) {
  const serviceName = nameOf<IDomainServices>().storage;
  services.addSingleton(serviceName, new FileSystemStorage());
}

export async function addSuggestionProviders(services: IServiceCollection) {
  services.addSingleton(
    nameOf<IDomainServices>().suggestionProviders,
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
    nameOf<IDomainServices>().fileWatcherDependencyCache,
    (container: IDomainServices) => new DependencyCache(container.providerNames)
  );
}

export function addSuggestionPackageCache(services: IServiceCollection) {
  services.addSingleton(
    nameOf<IDomainServices>().packageCache,
    (container: IDomainServices) => new PackageCache(container.providerNames)
  );
}

export function addShellCache(services: IServiceCollection) {
  const serviceName = nameOf<IDomainServices>().shellCache;
  services.addSingleton(serviceName, new MemoryExpiryCache(serviceName));
}

export function addGetSuggestionProviderUseCase(services: IServiceCollection) {
  const serviceName = nameOf<IDomainServices>().GetSuggestionProvider;
  services.addSingleton(
    serviceName,
    (container: IDomainServices) => new GetSuggestionProvider(container.suggestionProviders)
  );
}

export function addGetDependencyChangesUseCase(services: IServiceCollection) {
  const serviceName = nameOf<IDomainServices>().getDependencyChanges;
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
  const serviceName = nameOf<IDomainServices>().fetchPackages;
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
  const serviceName = nameOf<IDomainServices>().fetchPackage;
  services.addSingleton(
    serviceName,
    (container: IDomainServices) =>
      new FetchPackage(
        container.packageCache,
        container.loggerFactory.create(serviceName)
      )
  );
}