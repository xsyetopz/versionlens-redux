import type { IDomainServices } from '#domain';
import { CachingOptions } from '#domain/caching';
import { createJsonClient, HttpOptions } from '#domain/clients';
import type { IServiceCollection, IServiceProvider } from '#domain/di';
import type { IProviderServices } from '#domain/providers';
import {
  type IDenoServices,
  DenoConfig,
  DenoFeatures,
  DenoService,
  DenoSuggestionProvider,
  DenoSuggestionResolver,
  JsrClient
} from "#domain/providers/deno";
import { INpmServices } from '#domain/providers/npm';
import { nameOf } from '#domain/utils';

/**
 * Registers Deno caching options as a singleton.
 * @param services The service collection to add to.
 */
export function addCachingOptions(services: IServiceCollection) {
  services.addSingleton(
    DenoService.denoCachingOpts,
    (container: IDomainServices) =>
      new CachingOptions(
        container.appConfig,
        DenoFeatures.Caching,
        'caching'
      )
  );
}

/**
 * Registers Deno HTTP options as a singleton.
 * @param services The service collection to add to.
 */
export function addHttpOptions(services: IServiceCollection) {
  services.addSingleton(
    DenoService.denoHttpOpts,
    (container: IDomainServices) =>
      new HttpOptions(
        container.appConfig,
        DenoFeatures.Http,
        'http'
      )
  );
}

/**
 * Registers the Deno configuration as a singleton.
 * @param services The service collection to add to.
 */
export function addDenoConfig(services: IServiceCollection) {
  services.addSingleton(
    DenoService.denoConfig,
    (container: IDenoServices & IDomainServices) =>
      new DenoConfig(
        container.appConfig,
        container.denoCachingOpts,
        container.denoHttpOpts
      )
  );
}

/**
 * Registers the JSR client as a singleton.
 * @param services The service collection to add to.
 */
export function addJsrClient(services: IServiceCollection) {
  const serviceName = DenoService.jsrClient;
  services.addSingleton(
    serviceName,
    (container: IDenoServices & IDomainServices) =>
      new JsrClient(
        container.denoConfig,
        createJsonClient(
          container.authorizer,
          {
            caching: container.denoCachingOpts,
            http: container.denoHttpOpts
          }
        ),
        container.urlRequestCache,
        container.loggerFactory.create(serviceName)
      )
  );
}

/**
 * Registers the Deno suggestion resolver as a singleton.
 * @param services The service collection to add to.
 */
export function addDenoSuggestionResolver(services: IServiceCollection) {
  const serviceName = DenoService.denoClient;
  services.addSingleton(
    serviceName,
    (container: INpmServices & IDenoServices & IDomainServices) => {
      const npmServices = container.serviceProvider.getService('npm') as IServiceProvider
      return new DenoSuggestionResolver(
        container.denoConfig,
        container.jsrClient,
        npmServices.getService(nameOf<INpmServices>().npmSuggestionResolver),
        container.loggerFactory.create(serviceName)
      )
    }
  );
}

/**
 * Registers the Deno suggestion provider as a scoped service.
 * @param services The service collection to add to.
 */
export function addSuggestionProvider(services: IServiceCollection) {
  services.addScoped(
    nameOf<IProviderServices>().suggestionProvider,
    (container: IDenoServices & IDomainServices) => {
      const npmServices = container.serviceProvider.getService('npm') as IServiceProvider
      try { npmServices.getService('suggestionProvider') } catch (err) { }

      return new DenoSuggestionProvider(
        container.denoClient,
        container.denoConfig,
        npmServices.getService('suggestionProvider'),
        container.loggerFactory.create(DenoSuggestionProvider.name)
      )
    }
  );
}