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

/**
 * Registers Docker caching options as a singleton.
 * @param services The service collection to add to.
 */
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

/**
 * Registers Docker HTTP options as a singleton.
 * @param services The service collection to add to.
 */
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

/**
 * Registers the Docker configuration as a singleton.
 * @param services The service collection to add to.
 */
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

/**
 * Registers the Docker Hub client as a singleton.
 * @param services The service collection to add to.
 */
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

/**
 * Registers the Microsoft Docker client as a singleton.
 * @param services The service collection to add to.
 */
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

/**
 * Registers the Docker suggestion resolver as a singleton.
 * @param services The service collection to add to.
 */
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

/**
 * Registers the Docker suggestion provider as a scoped service.
 * @param services The service collection to add to.
 */
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