import { CachingOptions } from '#domain/caching';
import { HttpOptions } from '#domain/clients';
import { createHttpClient } from '#domain/clients/requestLight';
import { IServiceCollection } from '#domain/di';
import { IProviderServices } from '#domain/providers';
import {
  GoClient,
  GoConfig,
  GoFeatures,
  GoSuggestionProvider,
  IGoService
} from '#domain/providers/golang';
import { IDomainServices } from '#domain/services';
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

export function addHttpClient(services: IServiceCollection) {
  const serviceName = nameOf<IGoService>().goHttpClient;
  services.addSingleton(
    serviceName,
    (container: IGoService & IDomainServices) =>
      createHttpClient(
        {
          caching: container.goCachingOpts,
          http: container.goHttpOpts
        },
        container.logger.child({ logGroup: serviceName })
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
        container.logger.child({ logGroup: serviceName })
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
        container.logger.child({ logGroup: 'goSuggestionProvider' })
      )
  );
}