import type { IServiceCollection, IServiceProvider } from '#domain/di';
import {
  addCachingOptions,
  addHttpOptions,
  addNpmConfig,
  addNpmGitHubClient,
  addNpmSuggestionResolver,
  addNpmRegistryClient,
  addSuggestionProvider
} from '#domain/providers/npm';

/**
 * Configures the NPM service container by registering all necessary services.
 * @param serviceProvider The root service provider.
 * @param services The service collection to configure.
 * @returns A promise that resolves to the newly built child service provider.
 */
export async function configureContainer(
  serviceProvider: IServiceProvider,
  services: IServiceCollection
): Promise<IServiceProvider> {

  addCachingOptions(services);

  addHttpOptions(services);

  addNpmConfig(services);

  addNpmGitHubClient(services);

  addNpmRegistryClient(services);

  addNpmSuggestionResolver(services);

  addSuggestionProvider(services);

  return await services.buildChild("npm", serviceProvider);
}