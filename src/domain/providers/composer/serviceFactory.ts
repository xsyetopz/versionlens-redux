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
  ComposerService,
  ComposerSuggestionProvider,
  PackagistClient
} from '#domain/providers/composer';
import { nameOf } from '#domain/utils';

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

export function addJsonClient(services: IServiceCollection) {
  const serviceName = ComposerService.composerJsonClient;
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

export function addPackagistClient(services: IServiceCollection) {
  const serviceName = ComposerService.packagistClient;
  services.addSingleton(
    serviceName,
    (container: IComposerService & IDomainServices) =>
      new PackagistClient(
        container.composerConfig,
        container.composerJsonClient,
        container.loggerFactory.create(serviceName)
      )
  );
}

export function addComposerClient(services: IServiceCollection) {
  const serviceName = ComposerService.composerClient;
  services.addSingleton(
    serviceName,
    (container: IComposerService & IDomainServices) =>
      new ComposerClient(
        container.composerConfig,
        container.packagistClient,
        container.loggerFactory.create(serviceName)
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
        container.loggerFactory.create(ComposerSuggestionProvider.name)
      )
  );
}