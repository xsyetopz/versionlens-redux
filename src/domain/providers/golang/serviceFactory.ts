import type { IDomainServices } from '#domain';
import { CachingOptions } from '#domain/caching';
import { createHttpClient, HttpOptions } from '#domain/clients';
import type { IServiceCollection } from '#domain/di';
import type { IProviderServices } from '#domain/providers';
import {
  type IGoService,
  GoConfig,
  GoFeatures,
  GoHttpClient,
  GoService,
  GoSuggestionProvider,
  GoSuggestionResolver
} from '#domain/providers/golang';
import { nameOf } from '#domain/utils';

/**
 * Registers Go caching options as a singleton.
 * @param services The service collection to add to.
 */
export function addCachingOptions(services: IServiceCollection) {
  services.addSingleton(
    GoService.goCachingOpts,
    (container: IDomainServices) =>
      new CachingOptions(
        container.appConfig,
        GoFeatures.Caching,
        'caching'
      )
  );
}

/**
 * Registers Go HTTP options as a singleton.
 * @param services The service collection to add to.
 */
export function addHttpOptions(services: IServiceCollection) {
  services.addSingleton(
    GoService.goHttpOpts,
    (container: IDomainServices) =>
      new HttpOptions(
        container.appConfig,
        GoFeatures.Http,
        'http'
      )
  );
}

/**
 * Registers the Go configuration as a singleton.
 * @param services The service collection to add to.
 */
export function addGoConfig(services: IServiceCollection) {
  services.addSingleton(
    GoService.goConfig,
    (container: IGoService & IDomainServices) =>
      new GoConfig(
        container.appConfig,
        container.goCachingOpts,
        container.goHttpOpts
      )
  );
}

/**
 * Registers the Go HTTP client as a singleton.
 * @param services The service collection to add to.
 */
export function addGoHttpClient(services: IServiceCollection) {
  const serviceName = GoService.goHttpClient;
  services.addSingleton(
    serviceName,
    (container: IGoService & IDomainServices) =>
      new GoHttpClient(
        container.goConfig,
        createHttpClient(
          container.authorizer,
          {
            caching: container.goCachingOpts,
            http: container.goHttpOpts
          }
        ),
        container.urlRequestCache,
        container.loggerFactory.create(serviceName)
      )
  );
}

/**
 * Registers the Go suggestion resolver as a singleton.
 * @param services The service collection to add to.
 */
export function addGoSuggestionResolver(services: IServiceCollection) {
  const serviceName = GoService.goSuggestionResolver;
  services.addSingleton(
    serviceName,
    (container: IGoService & IDomainServices) =>
      new GoSuggestionResolver(
        container.goConfig,
        container.goHttpClient,
        container.loggerFactory.create(serviceName)
      )
  );
}

/**
 * Registers the Go suggestion provider as a scoped service.
 * @param services The service collection to add to.
 */
export function addSuggestionProvider(services: IServiceCollection) {
  services.addScoped(
    nameOf<IProviderServices>().suggestionProvider,
    (container: IGoService & IDomainServices) =>
      new GoSuggestionProvider(
        container.goSuggestionResolver,
        container.goConfig,
        container.loggerFactory.create(GoSuggestionProvider.name)
      )
  );
}