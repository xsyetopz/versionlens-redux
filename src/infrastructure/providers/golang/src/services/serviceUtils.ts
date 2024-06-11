import { CachingOptions } from '#domain/caching';
import { IServiceCollection } from '#domain/di';
import { HttpOptions } from '#domain/http';
import { IDomainServices, IProviderServices } from '#domain/services';
import { nameOf } from "domain/utils";
import { createHttpClient } from "infrastructure/http";
import { GoContributions } from "../definitions/eGoContributions";
import { GoClient } from "../goClient";
import { GoConfig } from "../goConfig";
import { GoSuggestionProvider } from "../goSuggestionProvider";
import { IGoService } from "./iGoServices";

export function addCachingOptions(services: IServiceCollection) {
  services.addSingleton(
    nameOf<IGoService>().goCachingOpts,
    (container: IDomainServices) =>
      new CachingOptions(
        container.appConfig,
        GoContributions.Caching,
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
        GoContributions.Http,
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