import { IServiceCollection, IServiceProvider } from '#domain/di';
import {
  addCachingOptions,
  addDockerClient,
  addDockerConfig,
  addDockerHubClient,
  addHttpOptions,
  addMicrosoftDockerClient,
  addSuggestionProvider
} from '#domain/providers/docker';

export async function configureContainer(
  serviceProvider: IServiceProvider,
  services: IServiceCollection
): Promise<IServiceProvider> {

  addCachingOptions(services);

  addHttpOptions(services);

  addDockerConfig(services);

  addDockerHubClient(services);

  addMicrosoftDockerClient(services);

  addDockerClient(services);

  addSuggestionProvider(services);

  return await services.buildChild("docker", serviceProvider);
}