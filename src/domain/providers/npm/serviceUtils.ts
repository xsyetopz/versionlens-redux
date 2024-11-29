import type { IDomainServices } from '#domain';
import { CachingOptions } from '#domain/caching';
import { createJsonClient, HttpOptions } from '#domain/clients';
import type { IServiceCollection } from '#domain/di';
import type { IProviderServices } from '#domain/providers';
import {
  type INpmServices,
  GitHubClient,
  GitHubOptions,
  NpmConfig,
  NpmFeatures,
  NpmPackageClient,
  NpmRegistryClient,
  NpmSuggestionProvider
} from '#domain/providers/npm';
import { nameOf } from '#domain/utils';
import NpmRegistryFetch from 'npm-registry-fetch';

export function addCachingOptions(services: IServiceCollection) {
  services.addSingleton(
    nameOf<INpmServices>().npmCachingOpts,
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
    nameOf<INpmServices>().npmHttpOpts,
    (container: IDomainServices) =>
      new HttpOptions(
        container.appConfig,
        NpmFeatures.Http,
        'http'
      )
  );
}

export function addGithubOptions(services: IServiceCollection) {
  services.addSingleton(
    nameOf<INpmServices>().npmGitHubOpts,
    (container: IDomainServices) =>
      new GitHubOptions(
        container.appConfig,
        NpmFeatures.Github,
        'github'
      )
  );
}

export function addNpmConfig(services: IServiceCollection) {
  services.addSingleton(
    nameOf<INpmServices>().npmConfig,
    (container: INpmServices & IDomainServices) =>
      new NpmConfig(
        container.appConfig,
        container.npmCachingOpts,
        container.npmHttpOpts,
        container.npmGitHubOpts
      )
  );
}

export function addJsonClient(services: IServiceCollection) {
  const serviceName = nameOf<INpmServices>().githubJsonClient;
  services.addSingleton(
    serviceName,
    (container: INpmServices & IDomainServices) =>
      createJsonClient(
        container.authorizer,
        {
          caching: container.npmCachingOpts,
          http: container.npmHttpOpts
        }
      )
  );
}

export function addGitHubClient(services: IServiceCollection) {
  const serviceName = nameOf<INpmServices>().githubClient;
  services.addSingleton(
    serviceName,
    (container: INpmServices & IDomainServices) =>
      new GitHubClient(
        container.npmConfig,
        container.githubJsonClient,
        container.logger.child({ logGroup: serviceName })
      )
  );
}

export function addNpmRegistryClient(services: IServiceCollection) {
  const serviceName = nameOf<INpmServices>().npmRegistryClient;
  services.addSingleton(
    serviceName,
    (container: INpmServices & IDomainServices) =>
      new NpmRegistryClient(
        NpmRegistryFetch,
        container.npmConfig,
        container.logger.child({ logGroup: serviceName })
      )
  );
}

export function addNpmPackageClient(services: IServiceCollection) {
  const serviceName = nameOf<INpmServices>().npmClient;
  services.addSingleton(
    serviceName,
    (container: INpmServices & IDomainServices) =>
      new NpmPackageClient(
        container.npmConfig,
        container.npmRegistryClient,
        container.githubClient,
        container.logger.child({ logGroup: serviceName })
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
        container.logger.child({ logGroup: 'npmSuggestionProvider' })
      )
  );
}