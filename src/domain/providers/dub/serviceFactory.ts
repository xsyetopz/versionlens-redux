import type { IDomainServices } from '#domain';
import { CachingOptions } from '#domain/caching';
import { createJsonClient, HttpOptions } from '#domain/clients';
import type { IServiceCollection } from '#domain/di';
import type { IProviderServices } from '#domain/providers';
import {
  type IDubServices,
  DubClient,
  DubConfig,
  DubFeatures,
  DubJsonClient,
  DubService,
  DubSuggestionProvider
} from '#domain/providers/dub';
import { nameOf } from '#domain/utils';

export function addCachingOptions(services: IServiceCollection) {
  services.addSingleton(
    DubService.dubCachingOpts,
    (container: IDomainServices) =>
      new CachingOptions(
        container.appConfig,
        DubFeatures.Caching,
        'caching'
      )
  );
}

export function addHttpOptions(services: IServiceCollection) {
  services.addSingleton(
    DubService.dubHttpOpts,
    (container: IDomainServices) =>
      new HttpOptions(
        container.appConfig,
        DubFeatures.Http,
        'http'
      )
  );
}

export function addDubConfig(services: IServiceCollection) {
  services.addSingleton(
    DubService.dubConfig,
    (container: IDubServices & IDomainServices) =>
      new DubConfig(
        container.appConfig,
        container.dubCachingOpts,
        container.dubHttpOpts
      )
  );
}

export function addDubJsonClient(services: IServiceCollection) {
  const serviceName = DubService.dubJsonClient;
  services.addSingleton(
    serviceName,
    (container: IDubServices & IDomainServices) =>
      new DubJsonClient(
        container.dubConfig,
        createJsonClient(
          container.authorizer,
          {
            caching: container.dubCachingOpts,
            http: container.dubHttpOpts
          }
        ),
        container.urlRequestCache,
        container.loggerFactory.create(serviceName)
      )
  );
}

export function addDubClient(services: IServiceCollection) {
  const serviceName = DubService.dubClient;
  services.addSingleton(
    serviceName,
    (container: IDubServices & IDomainServices) =>
      new DubClient(
        container.dubConfig,
        container.dubJsonClient,
        container.loggerFactory.create(serviceName)
      )
  );
}

export function addSuggestionProvider(services: IServiceCollection) {
  services.addScoped(
    nameOf<IProviderServices>().suggestionProvider,
    (container: IDubServices & IDomainServices) =>
      new DubSuggestionProvider(
        container.dubClient,
        container.dubConfig,
        container.loggerFactory.create(DubSuggestionProvider.name)
      )
  );
}