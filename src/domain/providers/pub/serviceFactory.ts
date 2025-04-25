import type { IDomainServices } from '#domain';
import { CachingOptions } from '#domain/caching';
import { createJsonClient, HttpOptions } from '#domain/clients';
import type { IServiceCollection } from '#domain/di';
import type { IProviderServices } from '#domain/providers';
import {
  type IPubServices,
  PubClient,
  PubConfig,
  PubFeatures,
  PubJsonClient,
  PubService,
  PubSuggestionProvider
} from '#domain/providers/pub';
import { nameOf } from '#domain/utils';

export function addCachingOptions(services: IServiceCollection) {
  services.addSingleton(
    PubService.pubCachingOpts,
    (container: IDomainServices) =>
      new CachingOptions(
        container.appConfig,
        PubFeatures.Caching,
        'caching'
      )
  );
}

export function addHttpOptions(services: IServiceCollection) {
  services.addSingleton(
    PubService.pubHttpOpts,
    (container: IDomainServices) =>
      new HttpOptions(
        container.appConfig,
        PubFeatures.Http,
        'http'
      )
  );
}

export function addPubConfig(services: IServiceCollection) {
  services.addSingleton(
    PubService.pubConfig,
    (container: IPubServices & IDomainServices) =>
      new PubConfig(
        container.appConfig,
        container.pubCachingOpts,
        container.pubHttpOpts
      )
  );
}

export function addPubJsonClient(services: IServiceCollection) {
  const serviceName = PubService.pubJsonClient;
  services.addSingleton(
    serviceName,
    (container: IPubServices & IDomainServices) =>
      new PubJsonClient(
        container.pubConfig,
        createJsonClient(
          container.authorizer,
          {
            caching: container.pubCachingOpts,
            http: container.pubHttpOpts
          }
        ),
        container.urlRequestCache,
        container.loggerFactory.create(serviceName)
      )
  );
}

export function addPubClient(services: IServiceCollection) {
  const serviceName = PubService.pubClient;
  services.addSingleton(
    serviceName,
    (container: IPubServices & IDomainServices) =>
      new PubClient(
        container.pubConfig,
        container.pubJsonClient,
        container.loggerFactory.create(serviceName)
      )
  );
}

export function addSuggestionProvider(services: IServiceCollection) {
  services.addScoped(
    nameOf<IProviderServices>().suggestionProvider,
    (container: IPubServices & IDomainServices) =>
      new PubSuggestionProvider(
        container.pubClient,
        container.pubConfig,
        container.loggerFactory.create(PubSuggestionProvider.name)
      )
  );
}