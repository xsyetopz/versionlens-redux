import { CachingOptions } from '#domain/caching';
import { IServiceCollection } from '#domain/di';
import { HttpOptions } from '#domain/http';
import { createJsonClient } from '#domain/http/requestLight';
import { IProviderServices } from '#domain/providers';
import {
  CargoConfig,
  CargoFeatures,
  CargoSuggestionProvider,
  CratesClient,
  ICargoService
} from "#domain/providers/cargo";
import { IDomainServices } from '#domain/services';
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
        {
          caching: container.cargoCachingOpts,
          http: container.cargoHttpOpts
        },
        container.logger.child({ logGroup: serviceName })
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
        container.logger.child({ logGroup: serviceName })
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
        container.logger.child({ logGroup: 'cargoSuggestionProvider' })
      )
  );
}