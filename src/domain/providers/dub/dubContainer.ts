import { IServiceCollection, IServiceProvider } from '#domain/di';
import {
  addCachingOptions,
  addDubConfig,
  addDubJsonClient,
  addDubSuggestionResolver,
  addHttpOptions,
  addSuggestionProvider
} from '#domain/providers/dub';

/**
 * Configures the Dub service container by registering all necessary services.
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

  addDubConfig(services);

  addDubJsonClient(services);

  addDubSuggestionResolver(services);

  addSuggestionProvider(services);

  return await services.buildChild("dub", serviceProvider);
}