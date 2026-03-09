import type { IServiceCollection, IServiceProvider } from '#domain/di';
import {
  addCachingOptions,
  addHttpOptions,
  addRubyConfig,
  addRubyHttpClient,
  addRubyGitHubClient,
  addRubySuggestionResolver,
  addSuggestionProvider
} from '#domain/providers/ruby';

/**
 * Configures the Ruby service container by registering all necessary services.
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

  addRubyConfig(services);

  addRubyHttpClient(services);

  addRubyGitHubClient(services);

  addRubySuggestionResolver(services);

  addSuggestionProvider(services);

  return await services.buildChild("ruby", serviceProvider);
}
