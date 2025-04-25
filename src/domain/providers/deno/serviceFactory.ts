import type { IDomainServices } from '#domain';
import { CachingOptions } from '#domain/caching';
import { createJsonClient, HttpOptions } from '#domain/clients';
import type { IServiceCollection, IServiceProvider } from '#domain/di';
import type { IProviderServices } from '#domain/providers';
import {
  type IDenoServices,
  DenoClient,
  DenoConfig,
  DenoFeatures,
  DenoService,
  DenoSuggestionProvider,
  JsrClient
} from "#domain/providers/deno";
import { INpmServices } from '#domain/providers/npm';
import { nameOf } from '#domain/utils';

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

export function addDenoClient(services: IServiceCollection) {
  const serviceName = DenoService.denoClient;
  services.addSingleton(
    serviceName,
    (container: INpmServices & IDenoServices & IDomainServices) => {
      const npmServices = container.serviceProvider.getService('npm') as IServiceProvider
      return new DenoClient(
        container.denoConfig,
        container.jsrClient,
        npmServices.getService('npmClient'),
        container.loggerFactory.create(serviceName)
      )
    }
  );
}

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