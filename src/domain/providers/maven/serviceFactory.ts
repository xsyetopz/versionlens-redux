import type { IDomainServices } from '#domain';
import { CachingOptions } from '#domain/caching';
import { createHttpClient, createShellClient, HttpOptions } from '#domain/clients';
import type { IServiceCollection } from '#domain/di';
import type { IProviderServices } from '#domain/providers';
import {
  type IMavenServices,
  MavenClient,
  MavenConfig,
  MavenFeatures,
  MavenHttpClient,
  MavenService,
  MavenSuggestionProvider,
  MvnCli
} from '#domain/providers/maven';
import { nameOf } from '#domain/utils';

export function addCachingOptions(services: IServiceCollection) {
  services.addSingleton(
    MavenService.mavenCachingOpts,
    (container: IDomainServices) =>
      new CachingOptions(
        container.appConfig,
        MavenFeatures.Caching,
        'caching'
      )
  );
}

export function addHttpOptions(services: IServiceCollection) {
  services.addSingleton(
    MavenService.mavenHttpOpts,
    (container: IDomainServices) =>
      new HttpOptions(
        container.appConfig,
        MavenFeatures.Http,
        'http'
      )
  );
}

export function addMavenConfig(services: IServiceCollection) {
  services.addSingleton(
    MavenService.mavenConfig,
    (container: IMavenServices & IDomainServices) =>
      new MavenConfig(
        container.appConfig,
        container.mavenCachingOpts,
        container.mavenHttpOpts
      )
  );
}

export function addMvnCliClient(services: IServiceCollection) {
  const serviceName = MavenService.mvnCli;
  services.addSingleton(
    serviceName,
    (container: IMavenServices & IDomainServices) =>
      new MvnCli(
        container.mavenConfig,
        createShellClient(
          container.shellCache,
          container.mavenCachingOpts,
          container.loggerFactory.create(serviceName)
        ),
        container.loggerFactory.create(serviceName)
      )
  );
}

export function addMavenHttpClient(services: IServiceCollection) {
  const serviceName = MavenService.mavenHttpClient;
  services.addSingleton(
    serviceName,
    (container: IMavenServices & IDomainServices) =>
      new MavenHttpClient(
        container.mavenConfig,
        createHttpClient(
          container.authorizer,
          {
            caching: container.mavenCachingOpts,
            http: container.mavenHttpOpts
          }
        ),
        container.urlRequestCache,
        container.loggerFactory.create(serviceName)
      )
  );
}

export function addMavenClient(services: IServiceCollection) {
  const serviceName = MavenService.mavenClient;
  services.addSingleton(
    serviceName,
    (container: IMavenServices & IDomainServices) =>
      new MavenClient(
        container.mavenConfig,
        container.mavenHttpClient,
        container.loggerFactory.create(serviceName)
      )
  );
}

export function addSuggestionProvider(services: IServiceCollection) {
  services.addScoped(
    nameOf<IProviderServices>().suggestionProvider,
    (container: IMavenServices & IDomainServices) =>
      new MavenSuggestionProvider(
        container.mavenClient,
        container.mvnCli,
        container.mavenConfig,
        container.loggerFactory.create(MavenSuggestionProvider.name)
      )
  );
}