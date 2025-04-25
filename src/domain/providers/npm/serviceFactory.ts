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
  NpmPackageClient,
  NpmRegistryClient,
  NpmService,
  NpmSuggestionProvider
} from '#domain/providers/npm';
import { nameOf } from '#domain/utils';
import NpmRegistryFetch from 'npm-registry-fetch';

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

export function addNpmPackageClient(services: IServiceCollection) {
  const serviceName = NpmService.npmClient;
  services.addSingleton(
    serviceName,
    (container: INpmServices & IDomainServices) =>
      new NpmPackageClient(
        container.npmConfig,
        container.npmRegistryClient,
        container.npmGithubClient,
        container.loggerFactory.create(serviceName)
      )
  );
}

export function addSuggestionProvider(services: IServiceCollection) {
  services.addScoped(
    nameOf<IProviderServices>().suggestionProvider,
    (container: INpmServices & IDomainServices) =>
      new NpmSuggestionProvider(
        container.npmClient,
        container.npmConfig,
        container.loggerFactory.create(NpmSuggestionProvider.name)
      )
  );
}