import { CachingOptions } from '#domain/caching';
import { IServiceCollection } from '#domain/di';
import { HttpOptions } from '#domain/http';
import { IDomainServices, IProviderServices } from '#domain/services';
import { nameOf } from "domain/utils";
import { createJsonClient } from "infrastructure/http";
import { DubContributions } from "../definitions/eDubContributions";
import { DubClient } from "../dubClient";
import { DubConfig } from "../dubConfig";
import { DubSuggestionProvider } from "../dubSuggestionProvider";
import { IDubServices } from "./iDubServices";

export function addCachingOptions(services: IServiceCollection) {
  services.addSingleton(
    nameOf<IDubServices>().dubCachingOpts,
    (container: IDomainServices) =>
      new CachingOptions(
        container.appConfig,
        DubContributions.Caching,
        'caching'
      )
  );
}

export function addHttpOptions(services: IServiceCollection) {
  services.addSingleton(
    nameOf<IDubServices>().dubHttpOpts,
    (container: IDomainServices) =>
      new HttpOptions(
        container.appConfig,
        DubContributions.Http,
        'http'
      )
  );
}

export function addDubConfig(services: IServiceCollection) {
  services.addSingleton(
    nameOf<IDubServices>().dubConfig,
    (container: IDubServices & IDomainServices) =>
      new DubConfig(
        container.appConfig,
        container.dubCachingOpts,
        container.dubHttpOpts
      )
  );
}

export function addJsonClient(services: IServiceCollection) {
  const serviceName = nameOf<IDubServices>().dubJsonClient;
  services.addSingleton(
    serviceName,
    (container: IDubServices & IDomainServices) =>
      createJsonClient(
        {
          caching: container.dubCachingOpts,
          http: container.dubHttpOpts
        },
        container.logger.child({ logGroup: serviceName })
      )
  );
}

export function addDubClient(services: IServiceCollection) {
  const serviceName = nameOf<IDubServices>().dubClient;
  services.addSingleton(
    serviceName,
    (container: IDubServices & IDomainServices) =>
      new DubClient(
        container.dubConfig,
        container.dubJsonClient,
        container.logger.child({ logGroup: serviceName })
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
        container.logger.child({ logGroup: 'dubSuggestionProvider' })
      )
  );
}