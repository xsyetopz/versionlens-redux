import type { IDomainServices } from '#domain';
import { CachingOptions } from '#domain/caching';
import { createJsonClient, HttpOptions } from '#domain/clients';
import type { IServiceCollection } from '#domain/di';
import type { IProviderServices } from '#domain/providers';
import {
  type ICargoServices,
  CargoClient,
  CargoConfig,
  CargoFeatures,
  CargoService,
  CargoSuggestionProvider,
  CratesClient
} from "#domain/providers/cargo";
import { nameOf } from '#domain/utils';

export function addCachingOptions(services: IServiceCollection) {
  services.addSingleton(
    CargoService.cargoCachingOpts,
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
    CargoService.cargoHttpOpts,
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
    CargoService.cargoConfig,
    (container: ICargoServices & IDomainServices) =>
      new CargoConfig(
        container.appConfig,
        container.cargoCachingOpts,
        container.cargoHttpOpts
      )
  );
}

export function addCratesClient(services: IServiceCollection) {
  const serviceName = CargoService.cratesClient;
  services.addSingleton(
    serviceName,
    (container: ICargoServices & IDomainServices) =>
      new CratesClient(
        container.cargoConfig,
        createJsonClient(
          container.authorizer,
          {
            caching: container.cargoCachingOpts,
            http: container.cargoHttpOpts
          }
        ),
        container.urlRequestCache,
        container.loggerFactory.create(serviceName)
      )
  );
}

export function addCargoClient(services: IServiceCollection) {
  const serviceName = CargoService.cargoClient;
  services.addSingleton(
    serviceName,
    (container: ICargoServices & IDomainServices) =>
      new CargoClient(
        container.cargoConfig,
        container.cratesClient,
        container.loggerFactory.create(serviceName)
      )
  );
}

export function addSuggestionProvider(services: IServiceCollection) {
  services.addScoped(
    nameOf<IProviderServices>().suggestionProvider,
    (container: ICargoServices & IDomainServices) =>
      new CargoSuggestionProvider(
        container.cargoClient,
        container.cargoConfig,
        container.loggerFactory.create(CargoSuggestionProvider.name)
      )
  );
}