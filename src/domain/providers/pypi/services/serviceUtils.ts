import { CachingOptions } from '#domain/caching';
import { IServiceCollection } from '#domain/di';
import { HttpOptions } from '#domain/http';
import { createHttpClient } from '#domain/http/requestLight';
import { IProviderServices } from '#domain/providers';
import {
  IPypiService,
  PypiClient,
  PypiConfig,
  PypiFeatures,
  PypiSuggestionProvider
} from '#domain/providers/pypi';
import { IDomainServices } from '#domain/services';
import { nameOf } from '#domain/utils';

export function addCachingOptions(services: IServiceCollection) {
  services.addSingleton(
    nameOf<IPypiService>().pypiCachingOpts,
    (container: IDomainServices) =>
      new CachingOptions(
        container.appConfig,
        PypiFeatures.Caching,
        'caching'
      )
  );
}

export function addHttpOptions(services: IServiceCollection) {
  services.addSingleton(
    nameOf<IPypiService>().pypiHttpOpts,
    (container: IDomainServices) =>
      new HttpOptions(
        container.appConfig,
        PypiFeatures.Http,
        'http'
      )
  );
}

export function addPypiConfig(services: IServiceCollection) {
  services.addSingleton(
    nameOf<IPypiService>().pypiConfig,
    (container: IPypiService & IDomainServices) =>
      new PypiConfig(
        container.appConfig,
        container.pypiCachingOpts,
        container.pypiHttpOpts
      )
  );
}

export function addHttpClient(services: IServiceCollection) {
  const serviceName = nameOf<IPypiService>().pypiHttpClient;
  services.addSingleton(
    serviceName,
    (container: IPypiService & IDomainServices) =>
      createHttpClient(
        {
          caching: container.pypiCachingOpts,
          http: container.pypiHttpOpts
        },
        container.logger.child({ logGroup: serviceName })
      )
  );
}

export function addPypiClient(services: IServiceCollection) {
  const serviceName = nameOf<IPypiService>().pypiClient;
  services.addSingleton(
    serviceName,
    (container: IPypiService & IDomainServices) =>
      new PypiClient(
        container.pypiConfig,
        container.pypiHttpClient,
        container.logger.child({ logGroup: serviceName })
      )
  );
}

export function addSuggestionProvider(services: IServiceCollection) {
  services.addScoped(
    nameOf<IProviderServices>().suggestionProvider,
    (container: IPypiService & IDomainServices) =>
      new PypiSuggestionProvider(
        container.pypiClient,
        container.pypiConfig,
        container.logger.child({ logGroup: 'pypiSuggestionProvider' })
      )
  );
}