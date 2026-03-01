import type { IDomainServices } from '#domain';
import { CachingOptions } from '#domain/caching';
import { HttpOptions } from '#domain/clients';
import type { IServiceCollection, IServiceProvider } from '#domain/di';
import type { IProviderServices } from '#domain/providers';
import {
  type IPnpmServices,
  PnpmConfig,
  PnpmFeatures,
  PnpmService,
  PnpmSuggestionProvider
} from '#domain/providers/pnpm';
import { nameOf } from '#domain/utils';

/**
 * Registers PNPM caching options as a singleton.
 * @param services The service collection to add to.
 */
export function addCachingOptions(services: IServiceCollection) {
  services.addSingleton(
    PnpmService.pnpmCachingOpts,
    (container: IDomainServices) =>
      new CachingOptions(
        container.appConfig,
        PnpmFeatures.Caching,
        'caching'
      )
  );
}

/**
 * Registers PNPM HTTP options as a singleton.
 * @param services The service collection to add to.
 */
export function addHttpOptions(services: IServiceCollection) {
  services.addSingleton(
    PnpmService.pnpmHttpOpts,
    (container: IDomainServices) =>
      new HttpOptions(
        container.appConfig,
        PnpmFeatures.Http,
        'http'
      )
  );
}

/**
 * Registers the PNPM configuration as a singleton.
 * @param services The service collection to add to.
 */
export function addPnpmConfig(services: IServiceCollection) {
  services.addSingleton(
    PnpmService.pnpmConfig,
    (container: IPnpmServices & IDomainServices) =>
      new PnpmConfig(
        container.appConfig,
        container.cachingOptions,
        container.httpOptions
      )
  );
}

/**
 * Registers the PNPM suggestion provider as a scoped service.
 * @param services The service collection to add to.
 */
export function addSuggestionProvider(services: IServiceCollection) {
  services.addScoped(
    nameOf<IProviderServices>().suggestionProvider,
    (container: IPnpmServices & IDomainServices) => {
      const npmServices = container.serviceProvider.getService('npm') as IServiceProvider
      try { npmServices.getService('suggestionProvider') } catch (err) { }
      return new PnpmSuggestionProvider(
        container.pnpmConfig,
        npmServices.getService('suggestionProvider'),
        container.loggerFactory.create(PnpmSuggestionProvider.name)
      );
    }
  );
}