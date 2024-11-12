import { CachingOptions } from '#domain/caching';
import { createHttpClient, createShellClient, HttpOptions } from '#domain/clients';
import { IServiceCollection } from '#domain/di';
import { IProviderServices } from '#domain/providers';
import {
  IMavenServices,
  MavenClient,
  MavenConfig,
  MavenFeatures,
  MavenSuggestionProvider,
  MvnCli
} from '#domain/providers/maven';
import { IDomainServices } from '#domain/services';
import { nameOf } from '#domain/utils';

export function addCachingOptions(services: IServiceCollection) {
  services.addSingleton(
    nameOf<IMavenServices>().mavenCachingOpts,
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
    nameOf<IMavenServices>().mavenHttpOpts,
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
    nameOf<IMavenServices>().mavenConfig,
    (container: IMavenServices & IDomainServices) =>
      new MavenConfig(
        container.appConfig,
        container.mavenCachingOpts,
        container.mavenHttpOpts
      )
  );
}

export function addProcessClient(services: IServiceCollection) {
  const serviceName = nameOf<IMavenServices>().mvnShellClient;
  services.addSingleton(
    serviceName,
    (container: IMavenServices & IDomainServices) =>
      createShellClient(
        container.shellCache,
        container.mavenCachingOpts,
        container.logger.child({ logGroup: serviceName })
      )
  );
}

export function addCliClient(services: IServiceCollection) {
  const serviceName = nameOf<IMavenServices>().mvnCli;
  services.addSingleton(
    serviceName,
    (container: IMavenServices & IDomainServices) =>
      new MvnCli(
        container.mavenConfig,
        container.mvnShellClient,
        container.logger.child({ logGroup: serviceName })
      )
  );
}

export function addHttpClient(services: IServiceCollection) {
  const serviceName = nameOf<IMavenServices>().mavenHttpClient;
  services.addSingleton(
    serviceName,
    (container: IMavenServices & IDomainServices) =>
      createHttpClient(
        {
          caching: container.mavenCachingOpts,
          http: container.mavenHttpOpts
        },
        container.logger.child({ logGroup: serviceName })
      )
  );
}

export function addMavenClient(services: IServiceCollection) {
  const serviceName = nameOf<IMavenServices>().mavenClient;
  services.addSingleton(
    serviceName,
    (container: IMavenServices & IDomainServices) =>
      new MavenClient(
        container.mavenConfig,
        container.mavenHttpClient,
        container.logger.child({ logGroup: serviceName })
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
        container.logger.child({ logGroup: 'mavenSuggestionProvider' })
      )
  );
}