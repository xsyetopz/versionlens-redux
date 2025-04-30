import type { IDomainServices } from '#domain';
import { CachingOptions } from '#domain/caching';
import { createJsonClient, HttpOptions } from '#domain/clients';
import type { IServiceCollection } from '#domain/di';
import type { IProviderServices } from '#domain/providers';
import {
  type IDockerServices,
  DockerConfig,
  DockerFeatures,
  DockerHubClient,
  DockerService,
  DockerSuggestionProvider,
  DockerSuggestionResolver,
  MicrosoftDockerClient
} from '#domain/providers/docker';
import { nameOf } from '#domain/utils';

export function addCachingOptions(services: IServiceCollection) {
  services.addSingleton(
    DockerService.dockerCachingOpts,
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
    DockerService.dockerHttpOpts,
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
    DockerService.dockerConfig,
    (container: IDockerServices & IDomainServices) =>
      new DockerConfig(
        container.appConfig,
        container.dockerCachingOpts,
        container.dockerHttpOpts
      )
  );
}

export function addDockerHubClient(services: IServiceCollection) {
  const serviceName = DockerService.dockerHubClient;
  services.addSingleton(
    serviceName,
    (container: IDockerServices & IDomainServices) =>
      new DockerHubClient(
        container.dockerConfig,
        createJsonClient(
          container.authorizer,
          {
            caching: container.dockerCachingOpts,
            http: container.dockerHttpOpts
          }
        ),
        container.urlRequestCache,
        container.loggerFactory.create(serviceName)
      )
  );
}

export function addMicrosoftDockerClient(services: IServiceCollection) {
  const serviceName = DockerService.microsoftDockerClient;
  services.addSingleton(
    serviceName,
    (container: IDockerServices & IDomainServices) =>
      new MicrosoftDockerClient(
        container.dockerConfig,
        createJsonClient(
          container.authorizer,
          {
            caching: container.dockerCachingOpts,
            http: container.dockerHttpOpts
          }
        ),
        container.urlRequestCache,
        container.loggerFactory.create(serviceName)
      )
  );
}

export function addDockerClient(services: IServiceCollection) {
  const serviceName = DockerService.dockerSuggestionResolver;
  services.addSingleton(
    serviceName,
    (container: IDockerServices & IDomainServices) =>
      new DockerSuggestionResolver(
        container.dockerConfig,
        container.dockerHubClient,
        container.microsoftDockerClient,
        container.loggerFactory.create(serviceName)
      )
  );
}

export function addSuggestionProvider(services: IServiceCollection) {
  services.addScoped(
    nameOf<IProviderServices>().suggestionProvider,
    (container: IDockerServices & IDomainServices) =>
      new DockerSuggestionProvider(
        container.dockerSuggestionResolver,
        container.dockerConfig,
        container.loggerFactory.create(DockerSuggestionProvider.name)
      )
  );
}