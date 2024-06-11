import { CachingOptions } from '#domain/caching';
import { IServiceCollection } from '#domain/di';
import { HttpOptions } from '#domain/http';
import { IDomainServices, IProviderServices } from '#domain/services';
import { nameOf } from "domain/utils";
import { createJsonClient } from "infrastructure/http";
import { ComposerClient } from "../composerClient";
import { ComposerConfig } from "../composerConfig";
import { ComposerSuggestionProvider } from "../composerSuggestionProvider";
import { ComposerContributions } from "../definitions/eComposerContributions";
import { IComposerService } from "./iComposerServices";

export function addCachingOptions(services: IServiceCollection) {
  services.addSingleton(
    nameOf<IComposerService>().composerCachingOpts,
    (container: IDomainServices) =>
      new CachingOptions(
        container.appConfig,
        ComposerContributions.Caching,
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
        ComposerContributions.Http,
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
        {
          caching: container.composerCachingOpts,
          http: container.composerHttpOpts
        },
        container.logger.child({ logGroup: serviceName })
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