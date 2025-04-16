import type { IDomainServices } from '#domain';
import { CachingOptions } from '#domain/caching';
import { createJsonClient, HttpOptions } from '#domain/clients';
import type { IServiceCollection } from '#domain/di';
import type { IProviderServices } from '#domain/providers';
import {
  type IDockerServices,
  DockerClient,
  DockerConfig,
  DockerFeatures,
  DockerHubClient,
  DockerSuggestionProvider
} from '#domain/providers/docker';
import { nameOf } from '#domain/utils';

export function addCachingOptions(services: IServiceCollection) {
  services.addSingleton(
    nameOf<IDockerServices>().dockerCachingOpts,
    (container: IDomainServices) =>
      new CachingOptions(
        container.appConfig,
        DockerFeatures.Caching,
        'caching'
      )
  );
}

export function addHttpOptions(services: IServiceCollection) {
  services.addSingleton(
    nameOf<IDockerServices>().dockerHttpOpts,
    (container: IDomainServices) =>
      new HttpOptions(
        container.appConfig,
        DockerFeatures.Http,
        'http'
      )
  );
}

export function addDockerConfig(services: IServiceCollection) {
  services.addSingleton(
    nameOf<IDockerServices>().dockerConfig,
    (container: IDockerServices & IDomainServices) =>
      new DockerConfig(
        container.appConfig,
        container.dockerCachingOpts,
        container.dockerHttpOpts
      )
  );
}

export function addJsonClient(services: IServiceCollection) {
  const serviceName = nameOf<IDockerServices>().dockerJsonClient;
  services.addSingleton(
    serviceName,
    (container: IDockerServices & IDomainServices) =>
      createJsonClient(
        container.authorizer,
        {
          caching: container.dockerCachingOpts,
          http: container.dockerHttpOpts
        }
      )
  );
}

export function addDockerHubClient(services: IServiceCollection) {
  const serviceName = nameOf<IDockerServices>().dockerHubClient;
  services.addSingleton(
    serviceName,
    (container: IDockerServices & IDomainServices) =>
      new DockerHubClient(
        container.dockerConfig,
        container.dockerJsonClient,
        container.loggerFactory.create(serviceName)
      )
  );
}

export function addDockerClient(services: IServiceCollection) {
  const serviceName = nameOf<IDockerServices>().dockerClient;
  services.addSingleton(
    serviceName,
    (container: IDockerServices & IDomainServices) =>
      new DockerClient(
        container.dockerConfig,
        container.dockerHubClient,
        container.loggerFactory.create(serviceName)
      )
  );
}

export function addSuggestionProvider(services: IServiceCollection) {
  services.addScoped(
    nameOf<IProviderServices>().suggestionProvider,
    (container: IDockerServices & IDomainServices) =>
      new DockerSuggestionProvider(
        container.dockerClient,
        container.dockerConfig,
        container.loggerFactory.create(DockerSuggestionProvider.name)
      )
  );
}