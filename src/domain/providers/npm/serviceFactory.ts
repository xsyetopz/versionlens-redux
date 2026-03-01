import type { IDomainServices } from '#domain';
import { CachingOptions } from '#domain/caching';
import { createJsonClient, GitHubJsonClient, HttpOptions } from '#domain/clients';
import type { IServiceCollection } from '#domain/di';
import type { IProviderServices } from '#domain/providers';
import {
  type INpmServices,
  NpmConfig,
  NpmFeatures,
  NpmGitHubClient,
  NpmRegistryClient,
  NpmService,
  NpmSuggestionProvider,
  NpmSuggestionResolver
} from '#domain/providers/npm';
import { nameOf } from '#domain/utils';
import NpmRegistryFetch from 'npm-registry-fetch';

/**
 * Registers NPM caching options as a singleton.
 * @param services The service collection to add to.
 */
export function addCachingOptions(services: IServiceCollection) {
  services.addSingleton(
    NpmService.npmCachingOpts,
    (container: IDomainServices) =>
      new CachingOptions(
        container.appConfig,
        NpmFeatures.Caching,
        'caching'
      )
  );
}

/**
 * Registers NPM HTTP options as a singleton.
 * @param services The service collection to add to.
 */
export function addHttpOptions(services: IServiceCollection) {
  services.addSingleton(
    NpmService.npmHttpOpts,
    (container: IDomainServices) =>
      new HttpOptions(
        container.appConfig,
        NpmFeatures.Http,
        'http'
      )
  );
}

/**
 * Registers the NPM configuration as a singleton.
 * @param services The service collection to add to.
 */
export function addNpmConfig(services: IServiceCollection) {
  services.addSingleton(
    NpmService.npmConfig,
    (container: INpmServices & IDomainServices) =>
      new NpmConfig(
        container.appConfig,
        container.npmCachingOpts,
        container.npmHttpOpts
      )
  );
}

/**
 * Registers the NPM GitHub client as a singleton.
 * @param services The service collection to add to.
 */
export function addNpmGitHubClient(services: IServiceCollection) {
  const serviceName = NpmService.npmGithubClient;
  services.addSingleton(
    serviceName,
    (container: INpmServices & IDomainServices) =>
      new NpmGitHubClient(
        container.npmConfig,
        new GitHubJsonClient(
          container.cachingOptions,
          createJsonClient(
            container.authorizer,
            {
              caching: container.npmCachingOpts,
              http: container.npmHttpOpts
            }
          ),
          container.urlRequestCache
        ),
        container.loggerFactory.create(serviceName)
      )
  );
}

/**
 * Registers the NPM registry client as a singleton.
 * @param services The service collection to add to.
 */
export function addNpmRegistryClient(services: IServiceCollection) {
  const serviceName = NpmService.npmRegistryClient;
  services.addSingleton(
    serviceName,
    (container: INpmServices & IDomainServices) =>
      new NpmRegistryClient(
        NpmRegistryFetch,
        container.npmConfig,
        container.urlRequestCache,
        container.loggerFactory.create(serviceName)
      )
  );
}

/**
 * Registers the NPM suggestion resolver as a singleton.
 * @param services The service collection to add to.
 */
export function addNpmSuggestionResolver(services: IServiceCollection) {
  const serviceName = NpmService.npmSuggestionResolver;
  services.addSingleton(
    serviceName,
    (container: INpmServices & IDomainServices) =>
      new NpmSuggestionResolver(
        container.npmConfig,
        container.npmRegistryClient,
        container.npmGithubClient,
        container.loggerFactory.create(serviceName)
      )
  );
}

/**
 * Registers the NPM suggestion provider as a scoped service.
 * @param services The service collection to add to.
 */
export function addSuggestionProvider(services: IServiceCollection) {
  services.addScoped(
    nameOf<IProviderServices>().suggestionProvider,
    (container: INpmServices & IDomainServices) =>
      new NpmSuggestionProvider(
        container.npmSuggestionResolver,
        container.npmConfig,
        container.loggerFactory.create(NpmSuggestionProvider.name)
      )
  );
}