import { CachingOptions } from '#domain/caching';
import { IServiceCollection } from '#domain/di';
import { HttpOptions } from '#domain/http';
import { IDomainServices, IProviderServices } from '#domain/services';
import { nameOf } from '#domain/utils';
import { createJsonClient } from '#infrastructure/http';
import {
  GitHubClient,
  GitHubOptions,
  INpmServices,
  NpmConfig,
  NpmFeatures,
  NpmPackageClient,
  NpmRegistryClient,
  NpmSuggestionProvider
} from '#providers/npm';
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
        {
          caching: container.npmCachingOpts,
          http: container.npmHttpOpts
        },
        container.logger.child({ logGroup: serviceName })
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