import type { IDomainServices } from '#domain';
import { CachingOptions } from '#domain/caching';
import { createHttpClient, HttpOptions } from '#domain/clients';
import type { IServiceCollection } from '#domain/di';
import type { IProviderServices } from '#domain/providers';
import {
  type IPypiServices,
  PypiConfig,
  PypiFeatures,
  PypiHttpClient,
  PypiService,
  PypiSuggestionProvider,
  PypiSuggestionResolver
} from '#domain/providers/pypi';
import { nameOf } from '#domain/utils';

/**
 * Registers PyPi caching options as a singleton.
 * @param services The service collection to add to.
 */
export function addCachingOptions(services: IServiceCollection) {
  services.addSingleton(
    PypiService.pypiCachingOpts,
    (container: IDomainServices) =>
      new CachingOptions(
        container.appConfig,
        PypiFeatures.Caching,
        'caching'
      )
  );
}

/**
 * Registers PyPi HTTP options as a singleton.
 * @param services The service collection to add to.
 */
export function addHttpOptions(services: IServiceCollection) {
  services.addSingleton(
    PypiService.pypiHttpOpts,
    (container: IDomainServices) =>
      new HttpOptions(
        container.appConfig,
        PypiFeatures.Http,
        'http'
      )
  );
}

/**
 * Registers the PyPi configuration as a singleton.
 * @param services The service collection to add to.
 */
export function addPypiConfig(services: IServiceCollection) {
  services.addSingleton(
    PypiService.pypiConfig,
    (container: IPypiServices & IDomainServices) =>
      new PypiConfig(
        container.appConfig,
        container.pypiCachingOpts,
        container.pypiHttpOpts
      )
  );
}

/**
 * Registers the PyPi HTTP client as a singleton.
 * @param services The service collection to add to.
 */
export function addPypiHttpClient(services: IServiceCollection) {
  const serviceName = PypiService.pypiHttpClient;
  services.addSingleton(
    serviceName,
    (container: IPypiServices & IDomainServices) =>
      new PypiHttpClient(
        container.pypiConfig,
        createHttpClient(
          container.authorizer,
          {
            caching: container.pypiCachingOpts,
            http: container.pypiHttpOpts
          }
        ),
        container.urlRequestCache,
        container.loggerFactory.create(serviceName)
      )
  );
}

/**
 * Registers the PyPi suggestion resolver as a singleton.
 * @param services The service collection to add to.
 */
export function addPypiSuggestionResolver(services: IServiceCollection) {
  const serviceName = PypiService.pypiSuggestionResolver;
  services.addSingleton(
    serviceName,
    (container: IPypiServices & IDomainServices) =>
      new PypiSuggestionResolver(
        container.pypiConfig,
        container.pypiHttpClient,
        container.loggerFactory.create(serviceName)
      )
  );
}

/**
 * Registers the PyPi suggestion provider as a scoped service.
 * @param services The service collection to add to.
 */
export function addSuggestionProvider(services: IServiceCollection) {
  services.addScoped(
    nameOf<IProviderServices>().suggestionProvider,
    (container: IPypiServices & IDomainServices) =>
      new PypiSuggestionProvider(
        container.pypiSuggestionResolver,
        container.pypiConfig,
        container.loggerFactory.create(PypiSuggestionProvider.name)
      )
  );
}