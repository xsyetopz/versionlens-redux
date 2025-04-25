import type { IDomainServices } from '#domain';
import { CachingOptions } from '#domain/caching';
import { createHttpClient, HttpOptions } from '#domain/clients';
import type { IServiceCollection } from '#domain/di';
import type { IProviderServices } from '#domain/providers';
import {
  type IGoService,
  GoClient,
  GoConfig,
  GoFeatures,
  GoHttpClient,
  GoSuggestionProvider
} from '#domain/providers/golang';
import { nameOf } from '#domain/utils';

export function addCachingOptions(services: IServiceCollection) {
  services.addSingleton(
    nameOf<IGoService>().goCachingOpts,
    (container: IDomainServices) =>
      new CachingOptions(
        container.appConfig,
        GoFeatures.Caching,
        'caching'
      )
  );
}

export function addHttpOptions(services: IServiceCollection) {
  services.addSingleton(
    nameOf<IGoService>().goHttpOpts,
    (container: IDomainServices) =>
      new HttpOptions(
        container.appConfig,
        GoFeatures.Http,
        'http'
      )
  );
}

export function addGoConfig(services: IServiceCollection) {
  services.addSingleton(
    nameOf<IGoService>().goConfig,
    (container: IGoService & IDomainServices) =>
      new GoConfig(
        container.appConfig,
        container.goCachingOpts,
        container.goHttpOpts
      )
  );
}

export function addGoHttpClient(services: IServiceCollection) {
  const serviceName = nameOf<IGoService>().goHttpClient;
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

export function addGoClient(services: IServiceCollection) {
  const serviceName = nameOf<IGoService>().goClient;
  services.addSingleton(
    serviceName,
    (container: IGoService & IDomainServices) =>
      new GoClient(
        container.goConfig,
        container.goHttpClient,
        container.loggerFactory.create(serviceName)
      )
  );
}

export function addSuggestionProvider(services: IServiceCollection) {
  services.addScoped(
    nameOf<IProviderServices>().suggestionProvider,
    (container: IGoService & IDomainServices) =>
      new GoSuggestionProvider(
        container.goClient,
        container.goConfig,
        container.loggerFactory.create(GoSuggestionProvider.name)
      )
  );
}