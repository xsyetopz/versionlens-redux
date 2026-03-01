import type { IDomainServices } from '#domain';
import { CachingOptions } from '#domain/caching';
import { createJsonClient, HttpOptions } from '#domain/clients';
import type { IServiceCollection } from '#domain/di';
import type { IProviderServices } from '#domain/providers';
import {
  type IDubServices,
  DubConfig,
  DubFeatures,
  DubJsonClient,
  DubService,
  DubSuggestionProvider,
  DubSuggestionResolver
} from '#domain/providers/dub';
import { nameOf } from '#domain/utils';

/**
 * Registers Dub caching options as a singleton.
 * @param services The service collection to add to.
 */
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

/**
 * Registers Dub HTTP options as a singleton.
 * @param services The service collection to add to.
 */
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

/**
 * Registers the Dub configuration as a singleton.
 * @param services The service collection to add to.
 */
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

/**
 * Registers the Dub JSON client as a singleton.
 * @param services The service collection to add to.
 */
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

/**
 * Registers the Dub suggestion resolver as a singleton.
 * @param services The service collection to add to.
 */
export function addDubSuggestionResolver(services: IServiceCollection) {
  const serviceName = DubService.dubSuggestionResolver;
  services.addSingleton(
    serviceName,
    (container: IDubServices & IDomainServices) =>
      new DubSuggestionResolver(
        container.dubConfig,
        container.dubJsonClient,
        container.loggerFactory.create(serviceName)
      )
  );
}

/**
 * Registers the Dub suggestion provider as a scoped service.
 * @param services The service collection to add to.
 */
export function addSuggestionProvider(services: IServiceCollection) {
  services.addScoped(
    nameOf<IProviderServices>().suggestionProvider,
    (container: IDubServices & IDomainServices) =>
      new DubSuggestionProvider(
        container.dubSuggestionResolver,
        container.dubConfig,
        container.loggerFactory.create(DubSuggestionProvider.name)
      )
  );
}