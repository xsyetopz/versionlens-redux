import type { IDomainServices } from '#domain';
import { CachingOptions } from '#domain/caching';
import { createHttpClient, createShellClient, HttpOptions } from '#domain/clients';
import type { IServiceCollection } from '#domain/di';
import type { IProviderServices } from '#domain/providers';
import {
  type IMavenServices,
  MavenConfig,
  MavenFeatures,
  MavenHttpClient,
  MavenService,
  MavenSuggestionProvider,
  MavenSuggestionResolver,
  MvnCli
} from '#domain/providers/maven';
import { nameOf } from '#domain/utils';

/**
 * Registers Maven caching options as a singleton.
 * @param services The service collection to add to.
 */
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

/**
 * Registers Maven HTTP options as a singleton.
 * @param services The service collection to add to.
 */
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

/**
 * Registers the Maven configuration as a singleton.
 * @param services The service collection to add to.
 */
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

/**
 * Registers the MvnCli client as a singleton.
 * @param services The service collection to add to.
 */
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

/**
 * Registers the Maven HTTP client as a singleton.
 * @param services The service collection to add to.
 */
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

/**
 * Registers the Maven suggestion resolver as a singleton.
 * @param services The service collection to add to.
 */
export function addMavenSuggestionResolver(services: IServiceCollection) {
  const serviceName = MavenService.mavenSuggestionResolver;
  services.addSingleton(
    serviceName,
    (container: IMavenServices & IDomainServices) =>
      new MavenSuggestionResolver(
        container.mavenConfig,
        container.mavenHttpClient,
        container.loggerFactory.create(serviceName)
      )
  );
}

/**
 * Registers the Maven suggestion provider as a scoped service.
 * @param services The service collection to add to.
 */
export function addSuggestionProvider(services: IServiceCollection) {
  services.addScoped(
    nameOf<IProviderServices>().suggestionProvider,
    (container: IMavenServices & IDomainServices) =>
      new MavenSuggestionProvider(
        container.mavenSuggestionResolver,
        container.mvnCli,
        container.mavenConfig,
        container.loggerFactory.create(MavenSuggestionProvider.name)
      )
  );
}