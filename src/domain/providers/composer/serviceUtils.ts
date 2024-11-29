import type { IDomainServices } from '#domain';
import { CachingOptions } from '#domain/caching';
import { createJsonClient, HttpOptions } from '#domain/clients';
import type { IServiceCollection } from '#domain/di';
import type { IProviderServices } from '#domain/providers';
import {
  type IComposerService,
  ComposerClient,
  ComposerConfig,
  ComposerFeatures,
  ComposerSuggestionProvider
} from '#domain/providers/composer';
import { nameOf } from '#domain/utils';

export function addCachingOptions(services: IServiceCollection) {
  services.addSingleton(
    nameOf<IComposerService>().composerCachingOpts,
    (container: IDomainServices) =>
      new CachingOptions(
        container.appConfig,
        ComposerFeatures.Caching,
        'caching'
      )
  );
}

export function addHttpOptions(services: IServiceCollection) {
  services.addSingleton(
    nameOf<IComposerService>().composerHttpOpts,
    (container: IDomainServices) =>
      new HttpOptions(
        container.appConfig,
        ComposerFeatures.Http,
        'http'
      )
  );
}

export function addComposerConfig(services: IServiceCollection) {
  services.addSingleton(
    nameOf<IComposerService>().composerConfig,
    (container: IComposerService & IDomainServices) =>
      new ComposerConfig(
        container.appConfig,
        container.composerCachingOpts,
        container.composerHttpOpts
      )
  );
}

export function addJsonClient(services: IServiceCollection) {
  const serviceName = nameOf<IComposerService>().composerJsonClient;
  services.addSingleton(
    serviceName,
    (container: IComposerService & IDomainServices) =>
      createJsonClient(
        container.authorizer,
        {
          caching: container.composerCachingOpts,
          http: container.composerHttpOpts
        }
      )
  );
}

export function addComposerClient(services: IServiceCollection) {
  const serviceName = nameOf<IComposerService>().composerClient;
  services.addSingleton(
    serviceName,
    (container: IComposerService & IDomainServices) =>
      new ComposerClient(
        container.composerConfig,
        container.composerJsonClient,
        container.logger.child({ logGroup: serviceName })
      )
  );
}

export function addSuggestionProvider(services: IServiceCollection) {
  services.addScoped(
    nameOf<IProviderServices>().suggestionProvider,
    (container: IComposerService & IDomainServices) =>
      new ComposerSuggestionProvider(
        container.composerClient,
        container.composerConfig,
        container.logger.child({ logGroup: 'composerSuggestionProvider' })
      )
  );
}