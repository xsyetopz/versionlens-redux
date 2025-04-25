import type { IServiceCollection, IServiceProvider } from '#domain/di';
import {
  addCachingOptions,
  addHttpOptions,
  addNpmConfig,
  addNpmGitHubClient,
  addNpmPackageClient,
  addNpmRegistryClient,
  addSuggestionProvider
} from '#domain/providers/npm';

export async function configureContainer(
  serviceProvider: IServiceProvider,
  services: IServiceCollection
): Promise<IServiceProvider> {

  addCachingOptions(services);

  addHttpOptions(services);

  addNpmConfig(services);

  addNpmGitHubClient(services);

  addNpmRegistryClient(services);

  addNpmPackageClient(services);

  addSuggestionProvider(services);

  return await services.buildChild("npm", serviceProvider);
}