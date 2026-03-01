import type { IDomainServices } from '#domain';
import { CachingOptions } from '#domain/caching';
import { createJsonClient, HttpOptions } from '#domain/clients';
import type { IServiceCollection } from '#domain/di';
import type { IProviderServices } from '#domain/providers';
import {
  type ICargoServices,
  CargoConfig,
  CargoFeatures,
  CargoService,
  CargoSuggestionProvider,
  CargoSuggestionResolver,
  CratesClient
} from "#domain/providers/cargo";
import { nameOf } from '#domain/utils';

/**
 * Registers Cargo caching options as a singleton.
 * @param services The service collection to add to.
 */
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

/**
 * Registers Cargo HTTP options as a singleton.
 * @param services The service collection to add to.
 */
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

/**
 * Registers the Cargo configuration as a singleton.
 * @param services The service collection to add to.
 */
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

/**
 * Registers the Crates client as a singleton.
 * @param services The service collection to add to.
 */
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

/**
 * Registers the Cargo suggestion resolver as a singleton.
 * @param services The service collection to add to.
 */
export function addCargoSuggestionResolver(services: IServiceCollection) {
  const serviceName = CargoService.cargoSuggestionResolver;
  services.addSingleton(
    serviceName,
    (container: ICargoServices & IDomainServices) =>
      new CargoSuggestionResolver(
        container.cargoConfig,
        container.cratesClient,
        container.loggerFactory.create(serviceName)
      )
  );
}

/**
 * Registers the Cargo suggestion provider as a scoped service.
 * @param services The service collection to add to.
 */
export function addSuggestionProvider(services: IServiceCollection) {
  services.addScoped(
    nameOf<IProviderServices>().suggestionProvider,
    (container: ICargoServices & IDomainServices) =>
      new CargoSuggestionProvider(
        container.cargoSuggestionResolver,
        container.cargoConfig,
        container.loggerFactory.create(CargoSuggestionProvider.name)
      )
  );
}