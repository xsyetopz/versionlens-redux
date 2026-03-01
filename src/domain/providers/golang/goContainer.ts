import type { IServiceCollection, IServiceProvider } from '#domain/di';
import {
  addCachingOptions,
  addGoSuggestionResolver,
  addGoConfig,
  addGoHttpClient,
  addHttpOptions,
  addSuggestionProvider
} from '#domain/providers/golang';

/**
 * Configures the Go service container by registering all necessary services.
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

  addGoConfig(services);

  addGoHttpClient(services);

  addGoSuggestionResolver(services);

  addSuggestionProvider(services);

  return await services.buildChild("golang", serviceProvider);
}