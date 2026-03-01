import { IServiceCollection, IServiceProvider } from '#domain/di';
import {
  addCachingOptions,
  addHttpOptions,
  addPubConfig,
  addPubJsonClient,
  addPubSuggestionResolver,
  addSuggestionProvider
} from '#domain/providers/pub';

/**
 * Configures the Pub service container by registering all necessary services.
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

  addPubConfig(services);

  addPubJsonClient(services);

  addPubSuggestionResolver(services);

  addSuggestionProvider(services);

  return await services.buildChild("pub", serviceProvider);
}