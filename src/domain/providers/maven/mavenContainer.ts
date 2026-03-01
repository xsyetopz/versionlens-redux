import { IServiceCollection, IServiceProvider } from '#domain/di';
import {
  addCachingOptions,
  addHttpOptions,
  addMavenSuggestionResolver,
  addMavenConfig,
  addMavenHttpClient,
  addMvnCliClient,
  addSuggestionProvider
} from '#domain/providers/maven';

/**
 * Configures the Maven service container by registering all necessary services.
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

  addMavenConfig(services);

  addMvnCliClient(services);

  addMavenHttpClient(services);

  addMavenSuggestionResolver(services);

  addSuggestionProvider(services);

  return await services.buildChild("maven", serviceProvider);
}