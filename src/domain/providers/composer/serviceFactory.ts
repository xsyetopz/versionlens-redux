import type { IDomainServices } from '#domain';
import { CachingOptions } from '#domain/caching';
import { createJsonClient, HttpOptions } from '#domain/clients';
import type { IServiceCollection } from '#domain/di';
import type { IProviderServices } from '#domain/providers';
import {
  type IComposerService,
  ComposerConfig,
  ComposerFeatures,
  ComposerService,
  ComposerSuggestionProvider,
  ComposerSuggestionResolver,
  PackagistClient
} from '#domain/providers/composer';
import { nameOf } from '#domain/utils';

/**
 * Registers Composer caching options as a singleton.
 * @param services The service collection to add to.
 */
export function addCachingOptions(services: IServiceCollection) {
  services.addSingleton(
    ComposerService.composerCachingOpts,
    (container: IDomainServices) =>
      new CachingOptions(
        container.appConfig,
        ComposerFeatures.Caching,
        'caching'
      )
  );
}

/**
 * Registers Composer HTTP options as a singleton.
 * @param services The service collection to add to.
 */
export function addHttpOptions(services: IServiceCollection) {
  services.addSingleton(
    ComposerService.composerHttpOpts,
    (container: IDomainServices) =>
      new HttpOptions(
        container.appConfig,
        ComposerFeatures.Http,
        'http'
      )
  );
}

/**
 * Registers the Composer configuration as a singleton.
 * @param services The service collection to add to.
 */
export function addComposerConfig(services: IServiceCollection) {
  services.addSingleton(
    ComposerService.composerConfig,
    (container: IComposerService & IDomainServices) =>
      new ComposerConfig(
        container.appConfig,
        container.composerCachingOpts,
        container.composerHttpOpts
      )
  );
}

/**
 * Registers the Packagist client as a singleton.
 * @param services The service collection to add to.
 */
export function addPackagistClient(services: IServiceCollection) {
  const serviceName = ComposerService.packagistClient;
  services.addSingleton(
    serviceName,
    (container: IComposerService & IDomainServices) =>
      new PackagistClient(
        container.composerConfig,
        createJsonClient(
          container.authorizer,
          {
            caching: container.composerCachingOpts,
            http: container.composerHttpOpts
          }
        ),
        container.urlRequestCache,
        container.loggerFactory.create(serviceName)
      )
  );
}

/**
 * Registers the Composer suggestion resolver as a singleton.
 * @param services The service collection to add to.
 */
export function addComposerSuggestionResolver(services: IServiceCollection) {
  const serviceName = ComposerService.composerSuggestionResolver;
  services.addSingleton(
    serviceName,
    (container: IComposerService & IDomainServices) =>
      new ComposerSuggestionResolver(
        container.composerConfig,
        container.packagistClient,
        container.loggerFactory.create(serviceName)
      )
  );
}

/**
 * Registers the Composer suggestion provider as a scoped service.
 * @param services The service collection to add to.
 */
export function addSuggestionProvider(services: IServiceCollection) {
  services.addScoped(
    nameOf<IProviderServices>().suggestionProvider,
    (container: IComposerService & IDomainServices) =>
      new ComposerSuggestionProvider(
        container.composerSuggestionResolver,
        container.composerConfig,
        container.loggerFactory.create(ComposerSuggestionProvider.name)
      )
  );
}