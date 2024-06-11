import { IServiceCollection, IServiceProvider } from '#domain/di';
import {
  addCachingOptions,
  addGitHubClient,
  addGithubOptions,
  addHttpOptions,
  addJsonClient,
  addNpmConfig,
  addNpmPackageClient,
  addNpmRegistryClient,
  addSuggestionProvider
} from './services/serviceUtils';

export async function configureContainer(
  serviceProvider: IServiceProvider,
  services: IServiceCollection
): Promise<IServiceProvider> {

  addCachingOptions(services);

  addHttpOptions(services);

  addGithubOptions(services);

  addNpmConfig(services);

  addJsonClient(services);

  addGitHubClient(services);

  addNpmRegistryClient(services);

  addNpmPackageClient(services);

  addSuggestionProvider(services);

  return await services.buildChild("npm", serviceProvider);
}