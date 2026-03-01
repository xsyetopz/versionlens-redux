import type { IServiceCollection, IServiceProvider } from '#domain/di';
import {
  addCachingOptions,
  addCargoConfig,
  addCargoSuggestionResolver,
  addCratesClient,
  addHttpOptions,
  addSuggestionProvider
} from '#domain/providers/cargo';

/**
 * Configures the Cargo service container by registering all necessary services.
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

  addCargoConfig(services);

  addCratesClient(services);

  addCargoSuggestionResolver(services);

  addSuggestionProvider(services);

  return await services.buildChild("cargo", serviceProvider);
}