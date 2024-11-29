import type { IDomainServices } from '#domain';
import { CachingOptions } from '#domain/caching';
import { createHttpClient, HttpOptions } from '#domain/clients';
import type { IServiceCollection } from '#domain/di';
import type { IProviderServices } from '#domain/providers';
import {
  type IPypiService,
  PypiClient,
  PypiConfig,
  PypiFeatures,
  PypiSuggestionProvider
} from '#domain/providers/pypi';
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
        container.authorizer,
        {
          caching: container.pypiCachingOpts,
          http: container.pypiHttpOpts
        }
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