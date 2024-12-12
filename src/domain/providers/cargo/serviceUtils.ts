import type { IDomainServices } from '#domain';
import { CachingOptions } from '#domain/caching';
import { createJsonClient, HttpOptions } from '#domain/clients';
import type { IServiceCollection } from '#domain/di';
import type { IProviderServices } from '#domain/providers';
import {
  type ICargoService,
  CargoConfig,
  CargoFeatures,
  CargoSuggestionProvider,
  CratesClient
} from "#domain/providers/cargo";
import { nameOf } from '#domain/utils';

export function addCachingOptions(services: IServiceCollection) {
  services.addSingleton(
    nameOf<ICargoService>().cargoCachingOpts,
    (container: IDomainServices) =>
      new CachingOptions(
        container.appConfig,
        CargoFeatures.Caching,
        'caching'
      )
  );
}

export function addHttpOptions(services: IServiceCollection) {
  services.addSingleton(
    nameOf<ICargoService>().cargoHttpOpts,
    (container: IDomainServices) =>
      new HttpOptions(
        container.appConfig,
        CargoFeatures.Http,
        'http'
      )
  );
}

export function addCargoConfig(services: IServiceCollection) {
  services.addSingleton(
    nameOf<ICargoService>().cargoConfig,
    (container: ICargoService & IDomainServices) =>
      new CargoConfig(
        container.appConfig,
        container.cargoCachingOpts,
        container.cargoHttpOpts
      )
  );
}

export function addJsonClient(services: IServiceCollection) {
  const serviceName = nameOf<ICargoService>().cargoJsonClient;
  services.addSingleton(
    serviceName,
    (container: ICargoService & IDomainServices) =>
      createJsonClient(
        container.authorizer,
        {
          caching: container.cargoCachingOpts,
          http: container.cargoHttpOpts
        }
      )
  );
}

export function addCratesClient(services: IServiceCollection) {
  const serviceName = nameOf<ICargoService>().cratesClient;
  services.addSingleton(
    serviceName,
    (container: ICargoService & IDomainServices) =>
      new CratesClient(
        container.cargoConfig,
        container.cargoJsonClient,
        container.loggerFactory.create(serviceName)
      )
  );
}

export function addSuggestionProvider(services: IServiceCollection) {
  services.addScoped(
    nameOf<IProviderServices>().suggestionProvider,
    (container: ICargoService & IDomainServices) =>
      new CargoSuggestionProvider(
        container.cratesClient,
        container.cargoConfig,
        container.loggerFactory.create(CargoSuggestionProvider.name)
      )
  );
}